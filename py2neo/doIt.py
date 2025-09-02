import ast
import os
from neo4j import GraphDatabase

class DagVisitor(ast.NodeVisitor):
    def __init__(self):
        self.dag_name = None
        self.full_code = None  # To store the entire code for recreation
        self.tasks = {}  # task_var: {'task_id': str, 'type': str, 'code': str, 'sql': str or None}
        self.functions = {}  # func_name: {'code': str} for functions like myFunction_t12345
        self.dependencies = []  # list of (upstream_var, downstream_var)

    def visit_Assign(self, node):
        self.generic_visit(node)
        
        # Detect DAG creation: dag = DAG('dag_id', ...)
        if len(node.targets) == 1 and isinstance(node.targets[0], ast.Name) and node.targets[0].id == 'dag':
            if isinstance(node.value, ast.Call) and isinstance(node.value.func, ast.Name) and node.value.func.id == 'DAG':
                if node.value.args and isinstance(node.value.args[0], ast.Constant):
                    self.dag_name = node.value.args[0].value  # dag_id as string
        
        # Detect task assignments: var = Operator(task_id='task_id', ...) or var = module.Operator(...)
        elif len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            var_name = node.targets[0].id
            if isinstance(node.value, ast.Call):
                func = node.value.func
                is_operator = False
                task_type = None
                
                if isinstance(func, ast.Name) and 'Operator' in func.id:
                    is_operator = True
                    task_type = func.id
                elif isinstance(func, ast.Attribute) and 'Operator' in func.attr:
                    is_operator = True
                    task_type = func.attr
                
                if is_operator:
                    task_id = None
                    sql_code = None
                    for kw in node.value.keywords:
                        if kw.arg == 'task_id' and isinstance(kw.value, ast.Constant):
                            task_id = kw.value.value
                        elif kw.arg == 'sql' and isinstance(kw.value, ast.Constant):
                            sql_code = kw.value.value
                    if task_id:
                        self.tasks[var_name] = {
                            'task_id': task_id,
                            'type': task_type,
                            'code': ast.unparse(node),
                            'sql': sql_code
                        }

    def visit_FunctionDef(self, node):
        self.generic_visit(node)
        # Extract functions for table names in func names (e.g., myFunction_t12345)
        self.functions[node.name] = {'code': ast.unparse(node)}

    def visit_Expr(self, node):
        self.generic_visit(node)
        
        # Detect dependency chains: task1 >> task2 >> task3
        if isinstance(node.value, ast.BinOp) and isinstance(node.value.op, ast.RShift):
            self.extract_dependencies(node.value)

    def extract_dependencies(self, node):
        if isinstance(node, ast.BinOp) and isinstance(node.op, ast.RShift):
            # Recurse on left if it's also a chain
            if isinstance(node.left, ast.BinOp) and isinstance(node.left.op, ast.RShift):
                self.extract_dependencies(node.left)
            left_task = self.get_task_name(node.left)
            right_task = self.get_task_name(node.right)
            if left_task and right_task:
                self.dependencies.append((left_task, right_task))  # (upstream, downstream)

    def get_task_name(self, node):
        if isinstance(node, ast.Name):
            return node.id
        elif isinstance(node, ast.BinOp):
            # For chained, get the rightmost of the left subtree
            return self.get_task_name(node.right)
        return None

def store_in_neo4j(visitor, neo4j_uri, neo4j_user, neo4j_password, file_path):
    driver = GraphDatabase.driver(neo4j_uri, auth=(neo4j_user, neo4j_password))
    
    with driver.session() as session:
        # Create or merge DAG node with full code for recreation
        session.run(
            """
            MERGE (d:DAG {name: $dag_name})
            SET d.full_code = $full_code, d.file_path = $file_path
            """,
            dag_name=visitor.dag_name,
            full_code=visitor.full_code,
            file_path=file_path
        )
        
        # Create task nodes and relate to DAG
        for var_name, info in visitor.tasks.items():
            session.run(
                """
                MERGE (t:Task {task_id: $task_id})
                SET t.type = $type, t.code = $code, t.sql = $sql
                WITH t
                MATCH (d:DAG {name: $dag_name})
                MERGE (d)-[:HAS_TASK]->(t)
                """,
                task_id=info['task_id'],
                type=info['type'],
                code=info['code'],
                sql=info['sql'],
                dag_name=visitor.dag_name
            )
        
        # Create function (method) nodes and relate to DAG
        for func_name, info in visitor.functions.items():
            session.run(
                """
                MERGE (m:Method {name: $name})
                SET m.code = $code
                WITH m
                MATCH (d:DAG {name: $dag_name})
                MERGE (d)-[:HAS_METHOD]->(m)
                """,
                name=func_name,
                code=info['code'],
                dag_name=visitor.dag_name
            )
        
        # Create dependency relationships (upstream >> downstream means upstream -[:UPSTREAM_OF]-> downstream)
        for upstream_var, downstream_var in visitor.dependencies:
            upstream_task_id = visitor.tasks.get(upstream_var, {}).get('task_id')
            downstream_task_id = visitor.tasks.get(downstream_var, {}).get('task_id')
            if upstream_task_id and downstream_task_id:
                session.run(
                    """
                    MATCH (u:Task {task_id: $upstream})
                    MATCH (d:Task {task_id: $downstream})
                    MERGE (u)-[:UPSTREAM_OF]->(d)
                    """,
                    upstream=upstream_task_id,
                    downstream=downstream_task_id
                )

    driver.close()

# Configuration - adjust these as needed (or use environment variables)
dags_dir = "/dags"  # Directory containing DAG .py files (mounted volume)
neo4j_uri = "bolt://localhost:7687"  # Update with your Neo4j URI
neo4j_user = "neo4j"  # Update with your Neo4j username
neo4j_password = "password"  # Update with your Neo4j password

# Process each .py file in the directory
for filename in os.listdir(dags_dir):
    if filename.endswith(".py"):
        file_path = os.path.join(dags_dir, filename)
        with open(file_path, "r") as f:
            code = f.read()
        
        # Parse with AST
        try:
            tree = ast.parse(code)
        except SyntaxError as e:
            print(f"Syntax error in {file_path}: {e}")
            continue
        
        # Visit and extract
        visitor = DagVisitor()
        visitor.full_code = code  # Store full code for recreation
        visitor.visit(tree)
        
        # Store in Neo4j if DAG found
        if visitor.dag_name:
            store_in_neo4j(visitor, neo4j_uri, neo4j_user, neo4j_password, file_path)
            print(f"Successfully parsed and stored DAG '{visitor.dag_name}' from {file_path} in Neo4j.")
        else:
            print(f"No DAG found in {file_path}.")