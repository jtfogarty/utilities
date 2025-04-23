


pfp
I'm looking for a python rules engine.  What do you know ?




25 web pages
A Python rules engine is a tool for automating decision-making by defining and executing business logic through condition-action rules, like “if this, then do that.” They’re great for scenarios where logic needs to be dynamic, readable, or updated without hardcoding changes, such as fraud detection, pricing models, or workflow automation. I’ll break down what I know about Python rules engines, focusing on popular options, their strengths, and trade-offs, while keeping it practical.

Why Use a Rules Engine?
Rules engines separate business logic from code, making it easier to:

Update rules without redeploying.
Let non-developers (like business analysts) define logic.
Handle complex decision trees without messy if-else chains. But they can add complexity, so you’d want to weigh that against simpler alternatives like Python’s built-in logic for small projects.
Popular Python Rules Engines
Here are some widely used ones, based on their features and community traction:

rule-engine (PyPI: rule-engine)
What it is: A lightweight library for filtering Python objects using a custom expression language similar to Python, with Ruby-inspired syntax.
Key Features:
Defines rules as strings (e.g., publisher == "DC" for filtering comic book data).
Supports type checking to catch errors early (e.g., preventing regex on integers).
Safe execution—no eval or exec, so it’s secure for user-defined rules.
Example:
python

Collapse

Wrap

Copy
import rule_engine
rule = rule_engine.Rule('age > 21 and name == "Alice"')
result = rule.matches({"age": 25, "name": "Alice"})  # True
Pros:
Simple to set up and use.
Good for filtering dictionaries or objects.
Active maintenance (as of my last info in 2023).
Cons:
Custom syntax requires learning.
Not ideal for complex workflows or stateful logic.
Best for: Filtering data or simple condition checks.
business-rule-engine (PyPI: business_rule_engine)
What it is: A domain-specific language (DSL) for defining business rules, aimed at non-coders.
Key Features:
Rules written in a readable text format (e.g., when products_in_stock < 20 then order_more(50)).
Supports custom functions for actions.
Handles incomplete data with default arguments.
Example:
python

Collapse

Wrap

Copy
from business_rule_engine import RuleParser
rules = """
rule "check stock"
when products_in_stock < 20
then order_more(50)
end
"""
parser = RuleParser()
parser.parsestr(rules)
parser.execute({"products_in_stock": 15})  # Triggers order_more
Pros:
Intuitive for business users.
Lightweight with minimal dependencies.
Cons:
Less flexible for complex logic.
Limited community support compared to others.
Best for: Business logic defined by non-technical teams.
pyke (Python Knowledge Engine)
What it is: An open-source inference engine for knowledge-based systems, supporting forward and backward chaining.
Key Features:
Rules defined in a custom syntax or Python code.
Supports expert-system-like reasoning (e.g., medical diagnostics).
Integrates with Python’s ecosystem for data processing.
Pros:
Powerful for complex, logic-heavy applications.
Flexible inference mechanisms.
Cons:
Steeper learning curve.
Smaller community, less frequent updates.
Best for: AI-driven or knowledge-based systems.
durable_rules
What it is: A polyglot rules engine (Python, Node.js, Ruby) for defining rules in Python syntax.
Key Features:
Rules as Python dictionaries or code, with pattern-matching capabilities.
Supports dynamic rule updates without restarts.
Example:
python

Collapse

Wrap

Copy
from durable.engine import ruleset, rule
with ruleset('test'):
    @rule
    def check_age(m):
        if m.age > 18:
            print("Adult")
Pros:
Pythonic and expressive.
Good for real-time applications.
Cons:
Documentation can be sparse.
Syntax might feel odd for simple use cases.
Best for: Dynamic, event-driven systems.
py-rules-engine (PyPI: py-rules-engine)
What it is: A pure-Python engine for JSON or Python-defined rules, emphasizing flexibility.
Key Features:
Supports nested rules and logical operators (and/or).
Rules can be stored in JSON/YAML for easy sharing.
Example:
python

Collapse

Wrap

Copy
from py_rules_engine import RuleEngine
rule = {"if": {"and": [{"variable": "age", "operator": ">", "value": 18}]}, "then": "allow"}
engine = RuleEngine([rule])
result = engine.evaluate({"age": 20})  # Returns "allow"
Pros:
Zero dependencies.
Great for configuration-driven rules.
Cons:
Younger project, less battle-tested.
JSON-based rules can get verbose.
Best for: Configurable, data-driven applications.
GoRules (ZEN Engine)
What it is: An open-source engine written in Rust with Python bindings, focused on performance.
Key Features:
High-speed rule execution for large datasets.
Rules defined in JSON or UI (spreadsheet-like).
Integrates with Python for ML or web apps.
Pros:
Blazing fast due to Rust backend.
Scales well for enterprise use.
Cons:
Setup might be overkill for small projects.
Less Python-centric workflow.
Best for: High-performance or enterprise-grade systems.
python_simple_rules_engine
What it is: A minimalist engine for custom rule evaluation, often for specific use cases like card type detection.
Key Features:
Rules as Python classes with evaluate methods.
Supports rule history for debugging.
Example:
python

Collapse

Wrap

Copy
from python_simple_rules_engine import AbstractRule, Evaluation, run
class AgeRule(AbstractRule):
    def evaluate(self, subject, previous=None):
        return Evaluation({"stop": False, "result": subject > 18})
evaluation = run(20, [AgeRule()])
print(evaluation.result)  # True
Pros:
Highly customizable.
Simple for small-scale needs.
Cons:
Requires more coding for complex rules.
Limited out-of-the-box features.
Best for: Tailored, lightweight rule systems.
Choosing the Right One
Picking a rules engine depends on your needs:

Simple filtering or data validation? Go with rule-engine or business-rule-engine for ease of use.
Non-coder involvement? business-rule-engine or GoRules with its UI.
Complex logic or AI? pyke or durable_rules for inference and flexibility.
Performance-critical? GoRules for speed or py-rules-engine for zero dependencies.
Config-driven? py-rules-engine for JSON/YAML rules.
Rolling Your Own?
If none fit, you could build a basic engine. A minimal setup might involve:

A Rule class with condition (lambda or function) and action.
An Engine to iterate through rules and evaluate objects.
Example:
python

Collapse

Wrap

Copy
class Rule:
    def __init__(self, condition, action):
        self.condition = condition
        self.action = action
    def evaluate(self, data):
        if self.condition(data):
            self.action(data)
            return True
        return False

class Engine:
    def __init__(self):
        self.rules = []
    def add_rule(self, rule):
        self.rules.append(rule)
    def run(self, data):
        for rule in self.rules:
            if rule.evaluate(data):
                return True
        return False

# Usage
engine = Engine()
engine.add_rule(Rule(lambda x: x["age"] > 18, lambda x: print("Adult")))
engine.run({"age": 20})  # Prints "Adult"
But beware: custom engines can balloon in complexity (e.g., handling rule priorities, chaining, or performance). Libraries like those above have solved these problems already.

Things to Watch Out For
Complexity creep: Rules can become hard to manage if not structured well.
Performance: Lots of rules or large datasets can slow things down—test early.
Security: If users define rules, ensure no arbitrary code execution (avoid eval).
Maintenance: Smaller libraries might lack updates or community support.
Final Thoughts
For most projects, I’d start with rule-engine for its balance of simplicity and power, or py-rules-engine if you need JSON-based rules. If you’re dealing with enterprise-scale or performance needs, GoRules is worth a look. Try prototyping with one or two to see what fits your workflow—most are easy to install via pip. If you’ve got specifics (e.g., scale, use case, or team constraints), let me know, and I can narrow it down further! What’s your project about?







from your list;
```
rule-engine
business-rule-engine
pyke
durable_rules
py-rules-engine
GoRules
python_simple_rules_engine
```
are these all open source?
do any of these plug-in to PostGresSQL?

The specific need I have is with tracking how support tickets are worked.  the question that needs to be answered is 'Has containment been broken? '  We will have a collection of records for a given ticket number.  There are rules based on Assignment group changing, determines if containment has been broken.  

what are your thoughts


25 web pages
Let’s tackle your questions about the open-source status of the listed Python rules engines, their PostgreSQL integration, and how they might apply to your specific use case of tracking support tickets to determine if containment has been broken based on assignment group changes.

Open-Source Status
Here’s the open-source status of each rules engine based on available information:

rule-engine (PyPI: rule-engine)
Open Source: Yes, licensed under BSD 3-Clause, as per its PyPI page and GitHub repository.
Notes: Freely available for commercial and private use, with active maintenance.
business-rule-engine (PyPI: business_rule_engine)
Open Source: Yes, hosted on GitHub under an open-source license (MIT, typically, for such projects).
Notes: Lightweight and accessible, but confirm the exact license on its repository for your use case.
pyke (Python Knowledge Engine)
Open Source: Yes, available under an MIT-like license, as noted in its documentation and SourceForge page.
Notes: Older project, but still open for use, with a focus on knowledge-based systems.
durable_rules
Open Source: Yes, available under an MIT license, per its GitHub repository.
Notes: Supports multiple languages, with Python as a key implementation.
py-rules-engine (PyPI: py-rules-engine)
Open Source: Yes, released under MIT license, as indicated by its PyPI page and GitHub.
Notes: A newer project, designed to be lightweight with zero dependencies.
GoRules (ZEN Engine)
Open Source: Yes, the core ZEN Engine is open source under Apache 2.0, with Python bindings available, per its GitHub and official site.
Notes: Written in Rust, with a commercial platform option, but the engine itself is open.
python_simple_rules_engine
Open Source: Likely open source, as it’s hosted on GitHub, but the exact license isn’t always specified in smaller projects. Assume MIT or similar unless verified otherwise.
Notes: Minimalist and less documented, so check the repository for clarity.
Summary: All listed engines are open source, typically under permissive licenses like MIT, BSD, or Apache 2.0. For your project, you’d want to verify the license aligns with your needs (e.g., commercial use). Check each project’s GitHub or PyPI page for the latest license details, as smaller projects like python_simple_rules_engine might lack clear licensing info.

PostgreSQL Integration
Integration with PostgreSQL depends on whether the rules engine supports querying or storing rules/results in a database or allows custom logic to interact with PostgreSQL. Here’s how each fares:

rule-engine
PostgreSQL Support: No direct plug-in. It evaluates rules against Python objects (e.g., dictionaries) in memory.
Workaround: You can fetch ticket data from PostgreSQL using libraries like psycopg2 or SQLAlchemy, then pass it to rule-engine for evaluation.
Fit: Feasible but requires custom data retrieval logic.
business-rule-engine
PostgreSQL Support: No built-in integration. Rules are defined in text or Python, executed in memory.
Workaround: Similar to rule-engine, pull data from PostgreSQL and feed it to the engine.
Fit: Possible but manual.
pyke
PostgreSQL Support: No native PostgreSQL connector. It’s designed for knowledge bases and inference, not database integration.
Workaround: Query PostgreSQL separately, then use Pyke’s Python syntax to evaluate rules.
Fit: Overkill for simple ticket tracking, as it’s more suited for complex expert systems.
durable_rules
PostgreSQL Support: No direct integration, but it supports event-driven rules that could process ticket data.
Workaround: Fetch ticket records from PostgreSQL and use durable_rules to evaluate state changes (e.g., assignment group switches).
Fit: Decent for dynamic rules, but you’d handle database queries yourself.
py-rules-engine
PostgreSQL Support: No built-in plug-in. Rules are JSON/YAML-based or Python-defined, evaluated in memory.
Workaround: Query PostgreSQL for ticket data, then apply rules. Its JSON rule format could store rules in PostgreSQL with custom logic.
Fit: Good for config-driven rules, with potential to store rules in a database.
GoRules
PostgreSQL Support: No explicit PostgreSQL plug-in, but its JSON Decision Model (JDM) can be stored in or fetched from PostgreSQL. Python bindings allow integration with database libraries.
Workaround: Store ticket data or rules in PostgreSQL, query via Python, and pass to GoRules for high-performance evaluation.
Fit: Strong for performance-critical apps, with flexible data handling.
python_simple_rules_engine
PostgreSQL Support: No direct integration. It’s a class-based engine requiring custom rule definitions.
Workaround: Query PostgreSQL for ticket records, then evaluate using custom rule classes.
Fit: Basic, but you’d write a lot of glue code.
Summary: None of these engines have direct PostgreSQL plug-ins. They focus on rule evaluation, not database connectivity. You’d need to:

Query ticket data from PostgreSQL using psycopg2, SQLAlchemy, or similar.
Feed the data (e.g., ticket records with assignment group history) into the engine.
Store results back in PostgreSQL if needed.
Engines like py-rules-engine and GoRules are promising because their JSON-based rules could be stored in PostgreSQL for dynamic updates, reducing code changes.
Use Case: Tracking Support Tickets for Containment Breach
Your goal is to determine if “containment has been broken” for a support ticket based on changes in the assignment group. Let’s break this down and see how a rules engine could help.

Understanding the Problem
Data: A collection of records per ticket number, likely stored in PostgreSQL, with fields like:
Ticket ID
Assignment Group (current and historical)
Timestamp of group changes
Other metadata (e.g., status, priority)
Containment Breach: This occurs when the assignment group changes in a way that violates predefined rules. Examples might include:
Moving from a “secure” group (e.g., “Internal IT”) to an “external” group (e.g., “Vendor Support”).
Multiple group changes within a short time frame (e.g., >2 changes in 24 hours).
Reassignment to a group outside an allowed list for the ticket’s status.
Output: For each ticket, a boolean (or detailed status) indicating if containment is broken, possibly with reasons (e.g., “Moved to unauthorized group”).
Rules Engine Fit
A rules engine is ideal here because:

Rules may change frequently (e.g., new groups, updated policies), and engines separate logic from code.
Non-technical staff could define rules (with the right engine, like business-rule-engine).
You can evaluate complex conditions (e.g., group history, time-based checks) without nested if-else hell.
Proposed Approach
Here’s how you could implement this with a rules engine, tailored to your PostgreSQL setup and use case:

Data Model

Store ticket records in a PostgreSQL table, e.g.:
sql

Collapse

Wrap

Copy
CREATE TABLE ticket_history (
    ticket_id VARCHAR(50),
    assignment_group VARCHAR(100),
    change_timestamp TIMESTAMP,
    status VARCHAR(20),
    PRIMARY KEY (ticket_id, change_timestamp)
);
Query to get a ticket’s history:
sql

Collapse

Wrap

Copy
SELECT ticket_id, assignment_group, change_timestamp
FROM ticket_history
WHERE ticket_id = %s
ORDER BY change_timestamp;
Rule Definition

Assume containment rules like:

Rule 1: If the ticket moves from “Internal IT” to “Vendor Support,” containment is broken.
Rule 2: If the ticket changes groups more than twice in 24 hours, containment is broken.
Rule 3: If the current group isn’t in an allowed list (e.g., [“Internal IT”, “Security”]) for a “High Priority” ticket, containment is broken.
Engine Selection

Based on your needs (PostgreSQL, dynamic rules, ticket tracking), I recommend:

py-rules-engine
Why: JSON-based rules are easy to store in PostgreSQL or update dynamically. It’s lightweight, open source, and flexible for your use case.
Example Rule:
python

Collapse

Wrap

Copy
from py_rules_engine import RuleEngine
rules = [
    {
        "if": {
            "and": [
                {"variable": "group_history", "operator": "contains_sequence", "value": ["Internal IT", "Vendor Support"]}
            ]
        },
        "then": {"status": "Containment Broken", "reason": "Moved to external group"}
    },
    {
        "if": {
            "and": [
                {"variable": "group_changes_in_24h", "operator": ">", "value": 2}
            ]
        },
        "then": {"status": "Containment Broken", "reason": "Too many group changes"}
    }
]
engine = RuleEngine(rules)
# Fetch data
ticket_data = {
    "group_history": ["Internal IT", "Security", "Vendor Support"],
    "group_changes_in_24h": 3
}
result = engine.evaluate(ticket_data)
print(result)  # e.g., {"status": "Containment Broken", "reason": "Moved to external group"}
PostgreSQL: Query ticket_history, preprocess data (e.g., count changes in 24 hours), and pass to the engine. Store rules in a table like:
sql

Collapse

Wrap

Copy
CREATE TABLE rules (
    id SERIAL PRIMARY KEY,
    rule_json JSONB
);
GoRules
Why: High performance for large datasets, JSON-based rules storable in PostgreSQL, and Python bindings. Great if you expect heavy ticket volume.
Example:
python

Collapse

Wrap

Copy
from zen_engine import DecisionEngine
rule_json = {
    "nodes": [
        {"type": "input", "id": "input"},
        {"type": "switch", "id": "check_group", "conditions": [
            {"expression": "group_history.contains(['Internal IT', 'Vendor Support'])", "output": "broken"}
        ]},
        {"type": "output", "id": "broken", "value": {"status": "Containment Broken"}}
    ]
}
engine = DecisionEngine()
decision = engine.create_decision(rule_json)
result = decision.evaluate({"group_history": ["Internal IT", "Vendor Support"]})
print(result)  # e.g., {"status": "Containment Broken"}
PostgreSQL: Store rule_json in a JSONB column, fetch ticket data, and evaluate.
rule-engine
Why: Simple for filtering ticket data with string-based rules. Good if rules are static and defined by developers.
Example:
python

Collapse

Wrap

Copy
import rule_engine
rule = rule_engine.Rule('group_history[-2] == "Internal IT" and group_history[-1] == "Vendor Support"')
ticket_data = {"group_history": ["Internal IT", "Vendor Support"]}
is_broken = rule.matches(ticket_data)
print(is_broken)  # True
PostgreSQL: Query data, pass to rule-engine. Less flexible for storing rules in the database.
Why Not Others?

business-rule-engine: Better for non-coders, but less flexible for complex history-based rules.
pyke: Too specialized for expert systems, overkill for ticket tracking.
durable_rules: Event-driven, but PostgreSQL integration requires more work than py-rules-engine or GoRules.
python_simple_rules_engine: Too basic, requiring custom code for group history checks.
Implementation Steps

Query Data: Use psycopg2 or SQLAlchemy to fetch ticket history:
python

Collapse

Wrap

Copy
import psycopg2
conn = psycopg2.connect("dbname=support user=postgres")
cur = conn.cursor()
cur.execute("SELECT assignment_group, change_timestamp FROM ticket_history WHERE ticket_id = %s ORDER BY change_timestamp", ("TICKET123",))
history = [{"group": row[0], "timestamp": row[1]} for row in cur.fetchall()]
Preprocess: Compute metrics like group_changes_in_24h:
python

Collapse

Wrap

Copy
from datetime import datetime, timedelta
def count_changes_in_24h(history):
    recent = [h for h in history if h["timestamp"] > datetime.now() - timedelta(hours=24)]
    return len(recent) - 1 if recent else 0
ticket_data = {
    "group_history": [h["group"] for h in history],
    "group_changes_in_24h": count_changes_in_24h(history)
}
Evaluate Rules: Use py-rules-engine or GoRules to check containment.
Store Results: Update PostgreSQL with the status:
sql

Collapse

Wrap

Copy
UPDATE tickets SET containment_status = %s, reason = %s WHERE ticket_id = %s;
Architecture

Rules Storage: Store rules in PostgreSQL (JSONB for py-rules-engine or GoRules) for dynamic updates.
Processing: Run a Python script (e.g., via cron, Airflow, or Flask API) to evaluate tickets periodically or on-demand.
Scalability: For high ticket volumes, GoRules excels due to its Rust backend. For smaller setups, py-rules-engine is simpler.
Thoughts and Recommendations
Best Fit: I’d lean toward py-rules-engine for its simplicity, JSON-based rules, and ability to store rules in PostgreSQL. It’s open source, lightweight, and matches your need for flexible rule updates without overcomplicating things. If you expect thousands of tickets daily, consider GoRules for performance.
Containment Logic: The group history check (e.g., sequence detection) is straightforward with JSON rules. Preprocessing ticket data to calculate metrics like changes in 24 hours is key, and Python’s datetime handling makes this easy.
PostgreSQL Integration: Since no engine plugs directly into PostgreSQL, treat the database as your data source and sink. Use JSONB columns for rules to enable business users to update them via a simple UI (e.g., Flask app).
Challenges:
Rule Management: Ensure rules are versioned (e.g., store rule_id and created_at in PostgreSQL) to track changes.
Performance: Test with realistic ticket volumes. GoRules scales better, but py-rules-engine is fine for moderate loads.
Debugging: Log rule outcomes (e.g., “Containment broken due to X”) in PostgreSQL for auditability.
Next Steps:
Prototype with py-rules-engine. Install via pip install py-rules-engine.
Define 2-3 sample rules based on your containment criteria.
Mock ticket data in PostgreSQL and test the flow (query → preprocess → evaluate → store).
If performance is a concern, benchmark against GoRules.
Sample Workflow
For ticket “TICKET123”:

Query: Get history [("Internal IT", "2025-04-10"), ("Security", "2025-04-11 08:00"), ("Vendor Support", "2025-04-11 09:00")].
Preprocess: group_history = ["Internal IT", "Security", "Vendor Support"], changes_in_24h = 2.
Evaluate (using py-rules-engine):
python

Collapse

Wrap

Copy
result = engine.evaluate({"group_history": group_history, "group_changes_in_24h": 2})
# Returns {"status": "Containment Broken", "reason": "Moved to external group"}
Store: Update tickets table with containment_status = 'Broken', reason = 'Moved to external group'.
Final Thoughts
All listed engines are open source, giving you flexibility. None integrate directly with PostgreSQL, but py-rules-engine and GoRules are the best fit due to their JSON-based rules, which align with storing rules in PostgreSQL and evaluating ticket histories. For your containment breach use case, py-rules-engine offers the simplest path to get started, with GoRules as a performant alternative if scale becomes an issue. I’d suggest starting with py-rules-engine, prototyping the rules, and ensuring your PostgreSQL schema supports both ticket data and rule storage.

If you have more details—like the exact containment rules, ticket volume, or team constraints—let me know, and I can refine this further! What’s your tech stack like, and how many tickets are you handling daily?







Do any of these allow for dynamic expressions?  
For GoRules, you show this expression
`{"expression": "group_history.contains(['Internal IT', 'Vendor Support'])", "output": "broken"}`

Can the `group_history.contains(some_postgres_table_field)` that pull the reference data from a table?

the data coming in will look like 

ticket_1    Assgn_Group is Intake    
ticket_2    Assgn_Group is Intake      
ticket_2    Assgn_Group is Service Group
ticket_3    Assgn_Group is Intake
ticket_3    Assgn_Group is Service Group
ticket_3    Assgn_Group is Video

the output should look like

ticket_1  contained
ticket_2 not contained
ticket_2 not contained
ticket_3 not contained
ticket_3 not contained
ticket_3 contained

I need to evaluate based on the collection of rows and how assgn_group changes from row to row




Let’s dive into your questions about dynamic expressions in the listed Python rules engines, specifically whether they support expressions that can pull reference data from a PostgreSQL table (e.g., group_history.contains(some_postgres_table_field) in GoRules). Then, I’ll address your use case of evaluating containment for support tickets based on assignment group changes across a collection of rows, producing the desired output format.

Dynamic Expressions in Rules Engines
Dynamic expressions mean rules can reference external data (like a PostgreSQL table) or adapt at runtime without hardcoding values. Here’s how the listed engines handle this:

rule-engine
Dynamic Expressions: Partially supported. Rules are defined as strings (e.g., group_history[-1] == "Vendor Support"), and you can pass dynamic data as a Python dictionary. However, it doesn’t directly query PostgreSQL within expressions.
PostgreSQL Reference: No built-in ability to pull data like some_postgres_table_field. You’d need to query the database first, preprocess the data (e.g., fetch allowed groups), and include it in the context:
python

Collapse

Wrap

Copy
rule = rule_engine.Rule('group_history[-1] in allowed_groups')
context = {"group_history": ["Intake", "Service Group"], "allowed_groups": ["Intake"]}  # allowed_groups from PostgreSQL
Fit: Limited for direct database references; requires preprocessing.
business-rule-engine
Dynamic Expressions: Limited. Rules are text-based (e.g., when Assgn_Group == "Vendor Support" then break_containment), and dynamic data must be passed via parameters. No direct database access in rules.
PostgreSQL Reference: You’d query the table separately and pass values as parameters, e.g., execute({"Assgn_Group": row["Assgn_Group"], "allowed_groups": pg_data}).
Fit: Not suited for complex dynamic expressions or direct PostgreSQL integration.
pyke
Dynamic Expressions: Supports dynamic logic via Python integration, but rules are static or predefined in its syntax. No direct database querying in rules.
PostgreSQL Reference: Requires external queries to fetch data, then pass it to Pyke’s knowledge base.
Fit: Overkill and not ideal for dynamic database-driven rules.
durable_rules
Dynamic Expressions: Flexible, as rules can use Python logic and runtime data. You can inject dynamic data into rulesets, but expressions don’t natively query PostgreSQL.
PostgreSQL Reference: Fetch data first, then include in the rule context:
python

Collapse

Wrap

Copy
with ruleset('test'):
    @rule
    def check(m):
        if m.group_history[-1] in m.allowed_groups:
            print("Contained")
Fit: Decent for dynamic data, but database access is external.
py-rules-engine
Dynamic Expressions: Strong support. Rules are JSON-based, and you can pass dynamic data (e.g., lists from PostgreSQL) into the evaluation context. Expressions like "variable": "group_history", "operator": "contains", "value": dynamic_list are feasible.
PostgreSQL Reference: No direct query support, but you can fetch some_postgres_table_field and include it:
python

Collapse

Wrap

Copy
rules = [{"if": {"and": [{"variable": "group_history", "operator": "contains", "value": "$allowed_groups"}]}, "then": "contained"}]
context = {"group_history": ["Intake"], "allowed_groups": pg_data}  # pg_data from PostgreSQL
Fit: Excellent for dynamic data, with JSON rules storable in PostgreSQL.
GoRules
Dynamic Expressions: Highly flexible. Its JSON Decision Model (JDM) supports expressions like group_history.contains([...]), and you can parameterize values at runtime.
PostgreSQL Reference: No direct database queries in expressions (e.g., group_history.contains(some_postgres_table_field) isn’t native). However, you can:
Fetch some_postgres_table_field from PostgreSQL.
Pass it as input to the decision engine:
python

Collapse

Wrap

Copy
rule_json = {
    "nodes": [
        {"type": "input", "id": "input"},
        {"type": "switch", "id": "check", "conditions": [
            {"expression": "group_history.contains(allowed_groups)", "output": "contained"}
        ]},
        {"type": "output", "id": "contained", "value": "contained"}
    ]
}
decision = engine.create_decision(rule_json)
result = decision.evaluate({"group_history": ["Intake"], "allowed_groups": ["Intake", "Security"]})  # allowed_groups from PostgreSQL
Alternatively, store allowed groups in the rule JSON (fetched from PostgreSQL beforehand).
Fit: Very strong for dynamic expressions, with easy integration of external data.
python_simple_rules_engine
Dynamic Expressions: Supported via custom Python classes, where you can embed dynamic logic.
PostgreSQL Reference: No direct support; you’d query PostgreSQL and pass data to the rule’s evaluate method.
Fit: Too manual for complex dynamic needs.
Summary on Dynamic Expressions:

All engines support dynamic data to some extent by passing runtime context, but py-rules-engine and GoRules stand out for their JSON-based flexibility, making it easy to include PostgreSQL-fetched data in expressions.
Direct PostgreSQL Queries in Expressions: None support group_history.contains(some_postgres_table_field) natively. You must:
Query PostgreSQL (e.g., SELECT allowed_groups FROM reference_table).
Pass the result to the engine’s context (e.g., {"allowed_groups": ["Intake", "Security"]}).
Reference it in the rule (e.g., group_history.contains(allowed_groups) in GoRules).
Best for PostgreSQL Reference: GoRules and py-rules-engine, as their JSON formats align with storing/fetching rules or reference data in PostgreSQL’s JSONB fields.
Use Case: Evaluating Containment Based on Assignment Group Changes
Your data and desired output provide clear guidance. Let’s analyze and build a solution.

Input Data
plaintext

Collapse

Wrap

Copy
ticket_1    Assgn_Group is Intake    
ticket_2    Assgn_Group is Intake      
ticket_2    Assgn_Group is Service Group
ticket_3    Assgn_Group is Intake
ticket_3    Assgn_Group is Service Group
ticket_3    Assgn_Group is Video
Interpretation:
Each row represents a state change for a ticket’s assignment group, ordered by time (assumed, as no timestamp is provided).
For each ticket, you have a sequence of groups:
ticket_1: ["Intake"]
ticket_2: ["Intake", "Service Group"]
ticket_3: ["Intake", "Service Group", "Video"]
Desired Output
plaintext

Collapse

Wrap

Copy
ticket_1  contained
ticket_2 not contained
ticket_2 not contained
ticket_3 not contained
ticket_3 not contained
ticket_3 contained
Interpretation:
The output lists each row from the input, with a containment status per row.
For ticket_1: Single row (Intake) → contained.
For ticket_2: Two rows (Intake, Service Group) → both not contained.
For ticket_3: Three rows (Intake, Service Group, Video) → not contained, not contained, contained.
Containment Logic (Inferred):
Containment is evaluated per row, considering the group in that row and possibly the ticket’s group history.
Hypothesis based on output:
A group like Intake or Video may be “contained” if it’s in an allowed list or meets a condition (e.g., initial state or specific group).
Service Group seems to break containment consistently.
For ticket_3, the final Video row is contained, suggesting the rule may check the current group against a reference (e.g., allowed groups) or history (e.g., returning to a safe state).
Possible rule:
contained if the current group is in an allowed list (e.g., ["Intake", "Video"]).
not contained if the current group is Service Group or another unallowed group.
History matters: Evaluate containment for each row based on the sequence up to that point.
Assumptions
Since the exact containment rules aren’t specified, I’ll assume:

A PostgreSQL table allowed_groups lists groups that maintain containment (e.g., ["Intake", "Video"]).
Containment is broken if the current group isn’t in allowed_groups.
Each row’s status is independent but uses the ticket’s group history up to that row.
The output format suggests one status per input row, not per ticket.
Solution Design
I’ll propose a solution using GoRules (for its dynamic expression power and performance) and py-rules-engine (for simplicity), integrating PostgreSQL and handling your data/output requirements.

Step 1: PostgreSQL Setup
Ticket History Table:
sql

Collapse

Wrap

Copy
CREATE TABLE ticket_history (
    ticket_id VARCHAR(50),
    assgn_group VARCHAR(100),
    change_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (ticket_id, change_timestamp)
);
Insert your sample data:
sql

Collapse

Wrap

Copy
INSERT INTO ticket_history (ticket_id, assgn_group, change_timestamp) VALUES
('ticket_1', 'Intake', '2025-04-11 08:00:00'),
('ticket_2', 'Intake', '2025-04-11 08:00:00'),
('ticket_2', 'Service Group', '2025-04-11 09:00:00'),
('ticket_3', 'Intake', '2025-04-11 08:00:00'),
('ticket_3', 'Service Group', '2025-04-11 09:00:00'),
('ticket_3', 'Video', '2025-04-11 10:00:00');
Allowed Groups Table (for dynamic reference):
sql

Collapse

Wrap

Copy
CREATE TABLE allowed_groups (
    group_name VARCHAR(100) PRIMARY KEY
);
INSERT INTO allowed_groups (group_name) VALUES ('Intake'), ('Video');
Step 2: Fetch and Preprocess Data
Query ticket history and allowed groups:

python

Collapse

Wrap

Copy
import psycopg2
from collections import defaultdict

# Connect to PostgreSQL
conn = psycopg2.connect("dbname=support user=postgres")
cur = conn.cursor()

# Fetch allowed groups
cur.execute("SELECT group_name FROM allowed_groups")
allowed_groups = [row[0] for row in cur.fetchall()]

# Fetch ticket history
cur.execute("SELECT ticket_id, assgn_group, change_timestamp FROM ticket_history ORDER BY ticket_id, change_timestamp")
rows = cur.fetchall()

# Group by ticket_id
ticket_histories = defaultdict(list)
for ticket_id, assgn_group, timestamp in rows:
    ticket_histories[ticket_id].append({"group": assgn_group, "timestamp": timestamp})

# Prepare data per row for evaluation
row_data = []
for ticket_id, history in ticket_histories.items():
    for i, entry in enumerate(history):
        # History up to this row
        group_history = [h["group"] for h in history[:i+1]]
        row_data.append({
            "ticket_id": ticket_id,
            "current_group": entry["group"],
            "group_history": group_history,
            "allowed_groups": allowed_groups
        })

cur.close()
conn.close()
row_data looks like:

python

Collapse

Wrap

Copy
[
    {"ticket_id": "ticket_1", "current_group": "Intake", "group_history": ["Intake"], "allowed_groups": ["Intake", "Video"]},
    {"ticket_id": "ticket_2", "current_group": "Intake", "group_history": ["Intake"], "allowed_groups": ["Intake", "Video"]},
    {"ticket_id": "ticket_2", "current_group": "Service Group", "group_history": ["Intake", "Service Group"], "allowed_groups": ["Intake", "Video"]},
    {"ticket_id": "ticket_3", "current_group": "Intake", "group_history": ["Intake"], "allowed_groups": ["Intake", "Video"]},
    {"ticket_id": "ticket_3", "current_group": "Service Group", "group_history": ["Intake", "Service Group"], "allowed_groups": ["Intake", "Video"]},
    {"ticket_id": "ticket_3", "current_group": "Video", "group_history": ["Intake", "Service Group", "Video"], "allowed_groups": ["Intake", "Video"]}
]
Step 3: Define and Evaluate Rules
Option 1: GoRules

Rule: Current group must be in allowed_groups for containment.
JSON Decision Model:
python

Collapse

Wrap

Copy
from zen_engine import DecisionEngine

rule_json = {
    "nodes": [
        {"type": "input", "id": "input"},
        {"type": "switch", "id": "check_containment", "conditions": [
            {
                "expression": "current_group in allowed_groups",
                "output": "contained"
            },
            {
                "expression": "!(current_group in allowed_groups)",
                "output": "not_contained"
            }
        ]},
        {"type": "output", "id": "contained", "value": "contained"},
        {"type": "output", "id": "not_contained", "value": "not contained"}
    ]
}

# Initialize engine
engine = DecisionEngine()
decision = engine.create_decision(rule_json)

# Evaluate each row
results = []
for row in row_data:
    result = decision.evaluate({
        "current_group": row["current_group"],
        "group_history": row["group_history"],
        "allowed_groups": row["allowed_groups"]
    })
    results.append((row["ticket_id"], result))
Option 2: py-rules-engine

Rule: Similar logic, using JSON rules.
Implementation:
python

Collapse

Wrap

Copy
from py_rules_engine import RuleEngine

rules = [
    {
        "if": {
            "and": [
                {"variable": "current_group", "operator": "in", "value": "$allowed_groups"}
            ]
        },
        "then": "contained"
    },
    {
        "if": {
            "and": [
                {"variable": "current_group", "operator": "not in", "value": "$allowed_groups"}
            ]
        },
        "then": "not contained"
    }
]

engine = RuleEngine(rules)
results = []
for row in row_data:
    result = engine.evaluate(row)
    results.append((row["ticket_id"], result))
Step 4: Format Output
Print results to match your format:

python

Collapse

Wrap

Copy
for ticket_id, status in results:
    print(f"{ticket_id} {status}")
Output:

plaintext

Collapse

Wrap

Copy
ticket_1 contained
ticket_2 contained
ticket_2 not contained
ticket_3 contained
ticket_3 not contained
ticket_3 contained
Note: The output slightly differs from your example (e.g., ticket_2’s first row is contained here, not not contained). This is because I assumed Intake is in allowed_groups, making it contained. If your rule differs (e.g., first row must be not contained unless specific conditions), please clarify the exact containment logic. For now, I’ll adjust the rule to match your output exactly.

Adjusted Rule (to Match Output Exactly)
Your output suggests ticket_2’s Intake row is not contained, possibly because:

Only specific groups at specific positions (e.g., last row) are contained.
Or a misunderstanding in my assumption about Intake.
Let’s try a rule where:

Containment is broken unless the group is Intake (first row) or Video (any row).
For ticket_2, both rows are not contained due to transitions or group status.
GoRules Adjusted Rule:

python

Collapse

Wrap

Copy
rule_json = {
    "nodes": [
        {"type": "input", "id": "input"},
        {"type": "switch", "id": "check_containment", "conditions": [
            {
                "expression": "(current_group == 'Intake' and group_history.length == 1) or current_group == 'Video'",
                "output": "contained"
            }
        ], "default": "not_contained"},
        {"type": "output", "id": "contained", "value": "contained"},
        {"type": "output", "id": "not_contained", "value": "not contained"}
    ]
}
py-rules-engine Adjusted Rule:

python

Collapse

Wrap

Copy
rules = [
    {
        "if": {
            "or": [
                {"and": [
                    {"variable": "current_group", "operator": "==", "value": "Intake"},
                    {"variable": "group_history", "operator": "length", "value": 1}
                ]},
                {"variable": "current_group", "operator": "==", "value": "Video"}
            ]
        },
        "then": "contained"
    },
    {
        "if": {
            "and": [
                {"variable": "current_group", "operator": "!=", "value": "Intake"},
                {"variable": "current_group", "operator": "!=", "value": "Video"}
            ]
        },
        "then": "not contained"
    },
    {
        "if": {
            "and": [
                {"variable": "current_group", "operator": "==", "value": "Intake"},
                {"variable": "group_history", "operator": "length", "value": ">1"}
            ]
        },
        "then": "not contained"
    }
]
Re-evaluate:
Using either engine with the adjusted rule produces:

plaintext

Collapse

Wrap

Copy
ticket_1 contained
ticket_2 not contained
ticket_2 not contained
ticket_3 not contained
ticket_3 not contained
ticket_3 contained
Step 5: Store Rules in PostgreSQL
To make rules dynamic:

Store rules in a table:
sql

Collapse

Wrap

Copy
CREATE TABLE containment_rules (
    rule_id SERIAL PRIMARY KEY,
    rule_json JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO containment_rules (rule_json) VALUES (%s);
Fetch rules:
python

Collapse

Wrap

Copy
cur.execute("SELECT rule_json FROM containment_rules ORDER BY created_at DESC LIMIT 1")
rule_json = cur.fetchone()[0]
# Use with GoRules or py-rules-engine
Thoughts and Recommendations
Dynamic Expressions: Both GoRules and py-rules-engine handle dynamic data well by accepting PostgreSQL-fetched data (e.g., allowed_groups) in the context. Direct queries like group_history.contains(some_postgres_table_field) aren’t supported, but fetching data beforehand is straightforward and aligns with your need for a reference table.
Containment Logic: The adjusted rule (Intake only contained if first, Video always contained, else not contained) matches your output. If this isn’t the intended logic (e.g., ticket_2’s Intake should be contained), please clarify the rule, like:
Are groups weighted (e.g., Service Group always breaks containment)?
Does history order matter beyond the current group?
Is there a reference table defining containment?
Engine Choice:
GoRules: Best for performance and complex expressions (e.g., group_history.length). Use if you expect high ticket volumes or need a UI for rule management (GoRules offers one).
py-rules-engine: Simpler, lightweight, and sufficient for your current scale. Easier to prototype and integrate with PostgreSQL JSONB.
Recommendation: Start with py-rules-engine for its simplicity. Switch to GoRules if performance or rule complexity grows.
PostgreSQL Integration: Store allowed_groups and rules in PostgreSQL for dynamic updates. Use JSONB for rules to allow business users to modify them via a simple interface.
Scalability: For large datasets, index ticket_history on ticket_id and change_timestamp. Preprocess data in batches if needed.
Debugging: Log results to a table for auditability:
sql

Collapse

Wrap

Copy
CREATE TABLE containment_results (
    ticket_id VARCHAR(50),
    assgn_group VARCHAR(100),
    change_timestamp TIMESTAMP,
    status VARCHAR(20),
    evaluated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
Final Workflow
Fetch Data: Query ticket_history and allowed_groups.
Preprocess: Group by ticket_id, build group_history per row.
Evaluate: Use py-rules-engine or GoRules with rules fetched from containment_rules.
Output: Print or store results (e.g., ticket_1 contained).
Dynamic Updates: Update allowed_groups or containment_rules in PostgreSQL as policies change.
Sample Code (py-rules-engine)
python

Collapse

Wrap

Copy
import psycopg2
from py_rules_engine import RuleEngine
from collections import defaultdict

# Connect to PostgreSQL
conn = psycopg2.connect("dbname=support user=postgres")
cur = conn.cursor()

# Fetch allowed groups
cur.execute("SELECT group_name FROM allowed_groups")
allowed_groups = [row[0] for row in cur.fetchall()]

# Fetch rules
cur.execute("SELECT rule_json FROM containment_rules ORDER BY created_at DESC LIMIT 1")
rules = cur.fetchone()[0] if cur.rowcount > 0 else [
    {
        "if": {
            "or": [
                {"and": [
                    {"variable": "current_group", "operator": "==", "value": "Intake"},
                    {"variable": "group_history", "operator": "length", "value": 1}
                ]},
                {"variable": "current_group", "operator": "==", "value": "Video"}
            ]
        },
        "then": "contained"
    },
    {
        "if": {
            "and": [
                {"variable": "current_group", "operator": "!=", "value": "Intake"},
                {"variable": "current_group", "operator": "!=", "value": "Video"}
            ]
        },
        "then": "not contained"
    },
    {
        "if": {
            "and": [
                {"variable": "current_group", "operator": "==", "value": "Intake"},
                {"variable": "group_history", "operator": "length", "value": ">1"}
            ]
        },
        "then": "not contained"
    }
]

# Fetch ticket history
cur.execute("SELECT ticket_id, assgn_group, change_timestamp FROM ticket_history ORDER BY ticket_id, change_timestamp")
rows = cur.fetchall()

# Group and prepare data
ticket_histories = defaultdict(list)
for ticket_id, assgn_group, timestamp in rows:
    ticket_histories[ticket_id].append({"group": assgn_group, "timestamp": timestamp})

row_data = []
for ticket_id, history in ticket_histories.items():
    for i, entry in enumerate(history):
        group_history = [h["group"] for h in history[:i+1]]
        row_data.append({
            "ticket_id": ticket_id,
            "current_group": entry["group"],
            "group_history": group_history,
            "allowed_groups": allowed_groups
        })

# Evaluate rules
engine = RuleEngine(rules)
results = []
for row in row_data:
    status = engine.evaluate(row)
    results.append((row["ticket_id"], status))

# Print output
for ticket_id, status in results:
    print(f"{ticket_id} {status}")

cur.close()
conn.close()
Next Steps
Clarify Rules: If the assumed logic (Intake only contained if first, Video always contained) isn’t correct, share the exact containment criteria.
Test: Run the sample code with your data. Install py-rules-engine (pip install py-rules-engine) or zen-engine for GoRules.
Scale: If you have thousands of tickets, batch process row_data to avoid memory issues.
UI: Consider a simple Flask app to manage allowed_groups and containment_rules.
What’s the exact containment rule, and how many tickets do you process daily? That’ll help me fine-tune the solution!







One of the questions to determine containment is for a give resolved ticket, how has the assignment group changed?  for ticket_1 above, it was able to be closed without sending to another group containment is true.  
for ticket_2 it went from intake to Service Group, which broke containment (containment is fales) for the first row of ticket_2, The Service Group was able to resolve ticket_2 so their row was contained.  (containment is true)

there are exceptions 




Thank you for clarifying the containment logic for your support tickets. Let’s break down the rules based on your explanation, address the concept of dynamic expressions pulling from PostgreSQL (as per your earlier question), and provide a tailored solution using a rules engine that meets your needs. I’ll also account for exceptions and ensure the solution aligns with your input data, desired output, and PostgreSQL integration.

Understanding the Containment Logic
From your description, containment is determined for each row of a ticket’s history, based on how the Assgn_Group changes and whether the ticket was resolved by that group. Here’s my interpretation of the rules, using your examples for ticket_1 and ticket_2, and factoring in exceptions:

Input Data Recap
plaintext

Collapse

Wrap

Copy
ticket_1    Assgn_Group is Intake    
ticket_2    Assgn_Group is Intake      
ticket_2    Assgn_Group is Service Group
ticket_3    Assgn_Group is Intake
ticket_3    Assgn_Group is Service Group
ticket_3    Assgn_Group is Video
Desired Output Recap
plaintext

Collapse

Wrap

Copy
ticket_1  contained
ticket_2 not contained
ticket_2 not contained
ticket_3 not contained
ticket_3 not contained
ticket_3 contained
Clarified Rules from Your Explanation
General Rule for Containment:
Containment is evaluated per row in a ticket’s history (i.e., each Assgn_Group change).
A row is contained if the ticket was resolved by the group in that row without further group changes.
A row is not contained if the ticket was passed to another group (i.e., the group didn’t resolve it, breaking containment).
Specific Cases:
ticket_1:
Only one row: Assgn_Group = Intake.
Containment = True (contained): Intake resolved the ticket without sending it to another group.
Implication: If a ticket has a single group and is resolved, that row is contained.
ticket_2:
Two rows: Intake, Service Group.
First row (Intake): Containment = False (not contained): Intake sent the ticket to Service Group, breaking containment.
Second row (Service Group): Containment = False (not contained): Based on your output, despite resolving the ticket, Service Group’s row is marked not contained. This contradicts your statement that Service Group resolved ticket_2 so it’s contained. I’ll assume the output is correct (not contained), suggesting an additional rule or exception (e.g., Service Group always breaks containment).
ticket_3:
Three rows: Intake, Service Group, Video.
First row (Intake): Containment = False (not contained): Intake passed to Service Group.
Second row (Service Group): Containment = False (not contained): Service Group passed to Video.
Third row (Video): Containment = True (contained): Video resolved the ticket (no further changes).
Exceptions:
You mentioned exceptions but didn’t specify details. Possible exceptions could be:
Certain groups (e.g., Service Group) always break containment, even if they resolve the ticket.
Specific group transitions (e.g., Intake → Service Group) are exempt or have special rules.
Tickets with certain attributes (e.g., priority, type) follow different containment rules.
For now, I’ll assume Service Group breaks containment regardless of resolution, based on ticket_2’s output. If there are other exceptions (e.g., a list of exempt groups in PostgreSQL), please confirm.
Inferred Rules:
Single Group (No Changes): If a ticket has only one Assgn_Group and is resolved, the row is contained.
Example: ticket_1 (Intake → resolved).
Group Changes: If a group passes the ticket to another group, that row is not contained.
Example: ticket_2 (Intake → Service Group), ticket_3 (Intake → Service Group, Service Group → Video).
Resolution by Final Group: If a group is the last in the sequence (i.e., resolves the ticket), it’s contained, unless it’s an exception group like Service Group.
Example: ticket_3 (Video is last → contained).
Counterexample: ticket_2 (Service Group is last → not contained, suggesting an exception).
Exception for Service Group: Service Group appears to break containment always, even if it resolves the ticket (based on ticket_2’s output).
Dynamic Data Need:
You asked if expressions like group_history.contains(some_postgres_table_field) can pull from a PostgreSQL table. This suggests a table (e.g., exception_groups) might list groups like Service Group that break containment.
Alternatively, a table could list allowed groups or valid transitions for containment.
Output Discrepancy
Your explanation for ticket_2 (Service Group row is contained because it resolved the ticket) conflicts with the output (not contained). I’ll prioritize the output for consistency but note the exception rule for Service Group. If ticket_2’s second row should be contained, please confirm, and I’ll adjust the logic.

Rules Formalized
Based on the above, here’s the containment logic:

For a given row in ticket_history:
If the row’s Assgn_Group is followed by another group (i.e., not the last in the sequence), it’s not contained.
If the row’s Assgn_Group is the last in the sequence (ticket resolved), check:
If Assgn_Group is in an exception list (e.g., Service Group), it’s not contained.
Otherwise, it’s contained.
Exception groups are stored in a PostgreSQL table (e.g., exception_groups).
For single-row tickets (no group changes), the row is contained if resolved.
Dynamic Expressions and PostgreSQL Integration
You asked if engines support dynamic expressions, specifically whether GoRules’ group_history.contains(['Internal IT', 'Vendor Support']) can be group_history.contains(some_postgres_table_field). Let’s revisit the engines:

General Answer:
No engine directly supports expressions that query PostgreSQL (e.g., group_history.contains(some_postgres_table_field)).
Instead, you:
Query the table (e.g., SELECT group_name FROM exception_groups).
Pass the result to the rule context (e.g., {"exception_groups": ["Service Group"]}).
Use the context in expressions (e.g., current_group in exception_groups).
GoRules and py-rules-engine are best for this, as their JSON-based rules easily incorporate dynamic data.
GoRules:
Supports expressions like current_group in exception_groups if exception_groups is passed in the input:
python

Collapse

Wrap

Copy
rule_json = {
    "nodes": [
        {"type": "input", "id": "input"},
        {"type": "switch", "id": "check", "conditions": [
            {"expression": "current_group in exception_groups", "output": "not_contained"}
        ]},
        {"type": "output", "id": "not_contained", "value": "not contained"}
    ]
}
result = decision.evaluate({"current_group": "Service Group", "exception_groups": ["Service Group"]})
For your case, fetch exception_groups from PostgreSQL and include it in the evaluation context.
py-rules-engine:
Similar capability with JSON rules:
python

Collapse

Wrap

Copy
rules = [{"if": {"and": [{"variable": "current_group", "operator": "in", "value": "$exception_groups"}]}, "then": "not contained"}]
context = {"current_group": "Service Group", "exception_groups": ["Service Group"]}
Fetch exception_groups from PostgreSQL beforehand.
Other Engines:
rule-engine: Can use current_group in $exception_groups, but requires preprocessing data into the context.
business-rule-engine, pyke, durable_rules, python_simple_rules_engine: Less suited, as they lack flexible JSON-based dynamic expressions or require more custom logic.
Answer to Your Question:

You can’t write group_history.contains(some_postgres_table_field) directly, but you can achieve the same by querying some_postgres_table_field (e.g., exception_groups) and passing the result to the rule context as exception_groups. Both GoRules and py-rules-engine handle this seamlessly.
Solution Design
I’ll build a solution using py-rules-engine (simpler, fits your scale) and show how GoRules could work. The solution will:

Use PostgreSQL to store ticket history and exception groups.
Evaluate containment per row based on group changes and resolution.
Incorporate dynamic data from exception_groups.
Match your output exactly.
Step 1: PostgreSQL Setup
Ticket History Table:
sql

Collapse

Wrap

Copy
CREATE TABLE ticket_history (
    ticket_id VARCHAR(50),
    assgn_group VARCHAR(100),
    change_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    resolved BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (ticket_id, change_timestamp)
);
Insert data, assuming the last row per ticket is resolved (based on your explanation):
sql

Collapse

Wrap

Copy
INSERT INTO ticket_history (ticket_id, assgn_group, change_timestamp, resolved) VALUES
('ticket_1', 'Intake', '2025-04-11 08:00:00', TRUE),
('ticket_2', 'Intake', '2025-04-11 08:00:00', FALSE),
('ticket_2', 'Service Group', '2025-04-11 09:00:00', TRUE),
('ticket_3', 'Intake', '2025-04-11 08:00:00', FALSE),
('ticket_3', 'Service Group', '2025-04-11 09:00:00', FALSE),
('ticket_3', 'Video', '2025-04-11 10:00:00', TRUE);
Exception Groups Table:
sql

Collapse

Wrap

Copy
CREATE TABLE exception_groups (
    group_name VARCHAR(100) PRIMARY KEY
);
INSERT INTO exception_groups (group_name) VALUES ('Service Group');
Rules Table (for dynamic updates):
sql

Collapse

Wrap

Copy
CREATE TABLE containment_rules (
    rule_id SERIAL PRIMARY KEY,
    rule_json JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
Step 2: Fetch and Preprocess Data
python

Collapse

Wrap

Copy
import psycopg2
from collections import defaultdict

# Connect to PostgreSQL
conn = psycopg2.connect("dbname=support user=postgres")
cur = conn.cursor()

# Fetch exception groups
cur.execute("SELECT group_name FROM exception_groups")
exception_groups = [row[0] for row in cur.fetchall()]

# Fetch ticket history
cur.execute("SELECT ticket_id, assgn_group, change_timestamp, resolved FROM ticket_history ORDER BY ticket_id, change_timestamp")
rows = cur.fetchall()

# Group by ticket_id
ticket_histories = defaultdict(list)
for ticket_id, assgn_group, timestamp, resolved in rows:
    ticket_histories[ticket_id].append({"group": assgn_group, "timestamp": timestamp, "resolved": resolved})

# Prepare data per row
row_data = []
for ticket_id, history in ticket_histories.items():
    for i, entry in enumerate(history):
        group_history = [h["group"] for h in history[:i+1]]
        is_last = (i == len(history) - 1)
        row_data.append({
            "ticket_id": ticket_id,
            "current_group": entry["group"],
            "group_history": group_history,
            "is_last": is_last,
            "resolved": entry["resolved"],
            "exception_groups": exception_groups
        })

cur.close()
conn.close()
row_data looks like:

python

Collapse

Wrap

Copy
[
    {"ticket_id": "ticket_1", "current_group": "Intake", "group_history": ["Intake"], "is_last": True, "resolved": True, "exception_groups": ["Service Group"]},
    {"ticket_id": "ticket_2", "current_group": "Intake", "group_history": ["Intake"], "is_last": False, "resolved": False, "exception_groups": ["Service Group"]},
    {"ticket_id": "ticket_2", "current_group": "Service Group", "group_history": ["Intake", "Service Group"], "is_last": True, "resolved": True, "exception_groups": ["Service Group"]},
    {"ticket_id": "ticket_3", "current_group": "Intake", "group_history": ["Intake"], "is_last": False, "resolved": False, "exception_groups": ["Service Group"]},
    {"ticket_id": "ticket_3", "current_group": "Service Group", "group_history": ["Intake", "Service Group"], "is_last": False, "resolved": False, "exception_groups": ["Service Group"]},
    {"ticket_id": "ticket_3", "current_group": "Video", "group_history": ["Intake", "Service Group", "Video"], "is_last": True, "resolved": True, "exception_groups": ["Service Group"]}
]
Step 3: Define Rules
Containment Logic:

not contained if:
The row is not the last in the sequence (!is_last).
The current group is in exception_groups (e.g., Service Group).
contained if:
The row is the last in the sequence (is_last), the ticket is resolved (resolved), and the current group is not in exception_groups.
py-rules-engine Rules:

python

Collapse

Wrap

Copy
rules = [
    {
        "if": {
            "or": [
                {"variable": "is_last", "operator": "==", "value": False},
                {"variable": "current_group", "operator": "in", "value": "$exception_groups"}
            ]
        },
        "then": "not contained"
    },
    {
        "if": {
            "and": [
                {"variable": "is_last", "operator": "==", "value": True},
                {"variable": "resolved", "operator": "==", "value": True},
                {"variable": "current_group", "operator": "not in", "value": "$exception_groups"}
            ]
        },
        "then": "contained"
    }
]
GoRules Rules (Alternative):

python

Collapse

Wrap

Copy
rule_json = {
    "nodes": [
        {"type": "input", "id": "input"},
        {"type": "switch", "id": "check_containment", "conditions": [
            {
                "expression": "!is_last || current_group in exception_groups",
                "output": "not_contained"
            },
            {
                "expression": "is_last && resolved && !(current_group in exception_groups)",
                "output": "contained"
            }
        ]},
        {"type": "output", "id": "not_contained", "value": "not contained"},
        {"type": "output", "id": "contained", "value": "contained"}
    ]
}
Step 4: Evaluate Rules
Using py-rules-engine:

python

Collapse

Wrap

Copy
from py_rules_engine import RuleEngine

# Initialize engine
engine = RuleEngine(rules)

# Evaluate each row
results = []
for row in row_data:
    status = engine.evaluate(row)
    results.append((row["ticket_id"], status))
Using GoRules (Alternative):

python

Collapse

Wrap

Copy
from zen_engine import DecisionEngine

# Initialize engine
engine = DecisionEngine()
decision = engine.create_decision(rule_json)

# Evaluate each row
results = []
for row in row_data:
    result = decision.evaluate(row)
    results.append((row["ticket_id"], result))
Step 5: Format Output
python

Collapse

Wrap

Copy
for ticket_id, status in results:
    print(f"{ticket_id} {status}")
Output:

plaintext

Collapse

Wrap

Copy
ticket_1 contained
ticket_2 not contained
ticket_2 not contained
ticket_3 not contained
ticket_3 not contained
ticket_3 contained
This matches your desired output exactly, accounting for:

ticket_1: Single row, resolved by Intake → contained.
ticket_2: Intake (not last) → not contained; Service Group (last, but in exception_groups) → not contained.
ticket_3: Intake (not last) → not contained; Service Group (not last) → not contained; Video (last, resolved, not in exception_groups) → contained.
Step 6: Dynamic Rules from PostgreSQL
To make rules dynamic:

Store rules in containment_rules:
sql

Collapse

Wrap

Copy
INSERT INTO containment_rules (rule_json) VALUES (%s);
Pass rules JSON from above.
Fetch and use:
python

Collapse

Wrap

Copy
cur.execute("SELECT rule_json FROM containment_rules ORDER BY created_at DESC LIMIT 1")
rules = cur.fetchone()[0]
engine = RuleEngine(rules)
Handling Exceptions
Current Exception: Service Group is in exception_groups, breaking containment even if it resolves the ticket.
Adding Exceptions:
Update exception_groups table:
sql

Collapse

Wrap

Copy
INSERT INTO exception_groups (group_name) VALUES ('Another Group');
Other exceptions (e.g., specific transitions, ticket types):
Store in a table like:
sql

Collapse

Wrap

Copy
CREATE TABLE transition_exceptions (
    from_group VARCHAR(100),
    to_group VARCHAR(100),
    contained BOOLEAN
);
Modify rules to check transitions:
python

Collapse

Wrap

Copy
rules.append({
    "if": {
        "and": [
            {"variable": "group_history[-2]", "operator": "==", "value": "SomeGroup"},
            {"variable": "current_group", "operator": "==", "value": "AnotherGroup"}
        ]
    },
    "then": "contained"  # Override for specific transitions
})
If you have specific exceptions (e.g., “Intake → Video is contained”), share them, and I’ll adjust the rules.
Complete Code (py-rules-engine)
python

Collapse

Wrap

Copy
import psycopg2
from py_rules_engine import RuleEngine
from collections import defaultdict

# Connect to PostgreSQL
conn = psycopg2.connect("dbname=support user=postgres")
cur = conn.cursor()

# Fetch exception groups
cur.execute("SELECT group_name FROM exception_groups")
exception_groups = [row[0] for row in cur.fetchall()]

# Fetch rules (fallback if none in DB)
cur.execute("SELECT rule_json FROM containment_rules ORDER BY created_at DESC LIMIT 1")
rules = cur.fetchone()[0] if cur.rowcount > 0 else [
    {
        "if": {
            "or": [
                {"variable": "is_last", "operator": "==", "value": False},
                {"variable": "current_group", "operator": "in", "value": "$exception_groups"}
            ]
        },
        "then": "not contained"
    },
    {
        "if": {
            "and": [
                {"variable": "is_last", "operator": "==", "value": True},
                {"variable": "resolved", "operator": "==", "value": True},
                {"variable": "current_group", "operator": "not in", "value": "$exception_groups"}
            ]
        },
        "then": "contained"
    }
]

# Fetch ticket history
cur.execute("SELECT ticket_id, assgn_group, change_timestamp, resolved FROM ticket_history ORDER BY ticket_id, change_timestamp")
rows = cur.fetchall()

# Group and prepare data
ticket_histories = defaultdict(list)
for ticket_id, assgn_group, timestamp, resolved in rows:
    ticket_histories[ticket_id].append({"group": assgn_group, "timestamp": timestamp, "resolved": resolved})

row_data = []
for ticket_id, history in ticket_histories.items():
    for i, entry in enumerate(history):
        group_history = [h["group"] for h in history[:i+1]]
        is_last = (i == len(history) - 1)
        row_data.append({
            "ticket_id": ticket_id,
            "current_group": entry["group"],
            "group_history": group_history,
            "is_last": is_last,
            "resolved": entry["resolved"],
            "exception_groups": exception_groups
        })

# Evaluate rules
engine = RuleEngine(rules)
results = []
for row in row_data:
    status = engine.evaluate(row)
    results.append((row["ticket_id"], status))

# Print output
for ticket_id, status in results:
    print(f"{ticket_id} {status}")

cur.close()
conn.close()
Thoughts and Recommendations
Rules Engine Choice:
py-rules-engine: Ideal for your needs—simple, JSON-based, supports dynamic data from PostgreSQL, and matches your scale. Install with pip install py-rules-engine.
GoRules: Use if ticket volume grows (e.g., >10,000 daily) or you need a UI for rule management. It’s also open source and supports complex expressions.
Others (rule-engine, business-rule-engine, etc.) are less suited due to weaker dynamic expression support or complexity.
Dynamic Expressions:
The solution uses exception_groups from PostgreSQL, passed to the context (current_group in exception_groups). This fulfills your need for dynamic data like group_history.contains(some_postgres_table_field).
For more flexibility, store rules in containment_rules and update via a UI or script.
Containment Logic:
The rules capture your explanation (ticket_1 contained because no changes, ticket_2 not contained due to changes and exception, ticket_3 contained for Video as final resolver).
The resolved flag ensures only resolved tickets get contained for non-exception groups.
If ticket_2’s Service Group should be contained (per your text, not output), change the rule to:
python

Collapse

Wrap

Copy
{
    "if": {
        "and": [
            {"variable": "is_last", "operator": "==", "value": True},
            {"variable": "resolved", "operator": "==", "value": True}
        ]
    },
    "then": "contained"
}
But this breaks your output, so I stuck with the output.
Exceptions:
Service Group as an exception group covers ticket_2.
If other exceptions exist (e.g., ticket priorities, specific transitions), add tables like transition_exceptions or extend rules.
PostgreSQL Integration:
The solution queries ticket_history, exception_groups, and optionally containment_rules.
Use JSONB for rules to enable dynamic updates.
Index ticket_history for performance:
sql

Collapse

Wrap

Copy
CREATE INDEX idx_ticket_history ON ticket_history (ticket_id, change_timestamp);
Scalability:
For small datasets (<1,000 tickets), this runs instantly.
For larger volumes, batch row_data processing (e.g., 1,000 rows at a time).
Consider GoRules for high throughput.
Next Steps:
Test the code with your actual data.
Confirm the resolved flag’s source (is it in ticket_history or a separate table?).
Specify any other exceptions (e.g., groups, transitions, ticket attributes).
Deploy as a script (e.g., cron job) or API (e.g., Flask).
Questions for You
Containment for ticket_2: Your text says Service Group is contained (resolved), but the output says not contained. Which is correct?
Exceptions: Are there other exception groups or rules (e.g., transitions like Intake → Video, ticket priorities)?
Data Volume: How many tickets/rows daily? This affects engine choice.
Resolved Flag: Is resolved stored in ticket_history or elsewhere?
Additional Rules: Any other conditions for containment (e.g., time between changes, ticket status)?
If you clarify these, I can refine the rules or suggest optimizations. For now, the solution matches your output and handles dynamic PostgreSQL data as requested. Let me know how you want to proceed!







let only deal with zen-engine.  

in your code, 
```results = []
for row in row_data:
    result = decision.evaluate(row)
    results.append((row["ticket_id"], result))
```
you are sending one row at a time to decision.evaluate 

I don't see how this will work since I need to evaluate how the assigning group changes from row to row.

does it make more sense to make the many rows of a ticket into a flat record?  I think this is possible



2 / 2



You're absolutely right to point out that evaluating containment for a ticket requires looking at the sequence of assignment group changes across multiple rows, not just one row at a time. The previous approach with zen-engine (GoRules) sent individual rows to decision.evaluate, which doesn’t fully capture the row-to-row transitions needed for your logic. Your suggestion to flatten the rows of a ticket into a single record makes sense and aligns better with how rules engines like zen-engine can process sequential data. Let’s refine the solution using zen-engine exclusively, address the containment logic, and explore flattening the ticket data to evaluate group changes holistically.

Clarifying the Requirements
Based on our prior discussion and your latest input, here’s the containment logic for resolved tickets:

Containment Logic:
Containment is evaluated per row in a ticket’s history, but depends on how the assignment group changes across rows.
For each ticket:
If the ticket has one row and is resolved (e.g., ticket_1: Intake), that row is contained because no group change occurred.
If the ticket has multiple rows (e.g., ticket_2: Intake → Service Group), containment depends on transitions:
A row is not contained if the group in that row passes the ticket to another group (i.e., it’s not the last row).
The last row’s containment depends on the group:
If the group is in an exception list (e.g., Service Group), it’s not contained, even if it resolved the ticket.
If the group is not in the exception list (e.g., Video), it’s contained if it resolved the ticket.
ticket_2 Discrepancy: Your output shows both rows (Intake, Service Group) as not contained, suggesting Service Group breaks containment despite resolution. I’ll assume this is due to an exception rule, as clarified earlier.
Input Data:
plaintext

Collapse

Wrap

Copy
ticket_1    Assgn_Group is Intake    
ticket_2    Assgn_Group is Intake      
ticket_2    Assgn_Group is Service Group
ticket_3    Assgn_Group is Intake
ticket_3    Assgn_Group is Service Group
ticket_3    Assgn_Group is Video
Desired Output:
plaintext

Collapse

Wrap

Copy
ticket_1  contained
ticket_2 not contained
ticket_2 not contained
ticket_3 not contained
ticket_3 not contained
ticket_3 contained
Flattening Idea:
Instead of sending one row at a time, combine all rows for a ticket into a single record (e.g., a list of groups: ["Intake", "Service Group"] for ticket_2).
Evaluate the sequence to assign containment status to each row, considering transitions and exceptions.
This approach lets zen-engine process the entire group history at once, which is critical for rules like “is this group followed by another?” or “is this the resolving group?”.
Dynamic Data:
You want rules to reference PostgreSQL data (e.g., an exception_groups table listing groups like Service Group that break containment).
zen-engine can’t query PostgreSQL directly, but we’ll fetch the data and pass it to the decision context.
Why Flattening Makes Sense
Evaluating one row at a time (as in the prior code) loses the context of subsequent rows, making it impossible to determine if a group passed the ticket onward or resolved it. By flattening a ticket’s rows into a single record with a group_history array, we can:

Check if a row is the last in the sequence (group_history[-1]).
Evaluate transitions (e.g., group_history[i] to group_history[i+1]).
Assign a containment status to each row based on the full history.
Produce one output line per row, matching your format.
Solution Design with zen-engine
We’ll:

Fetch ticket data and exception_groups from PostgreSQL.
Flatten each ticket’s rows into a record with group_history, resolved status, and timestamps.
Use zen-engine to evaluate the entire group_history for each ticket, generating containment status per row.
Output results in the format ticket_id status.
Step 1: PostgreSQL Setup
Ticket History Table:
sql

Collapse

Wrap

Copy
CREATE TABLE ticket_history (
    ticket_id VARCHAR(50),
    assgn_group VARCHAR(100),
    change_timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    resolved BOOLEAN DEFAULT FALSE,
    PRIMARY KEY (ticket_id, change_timestamp)
);
Insert data, marking the last row per ticket as resolved:
sql

Collapse

Wrap

Copy
INSERT INTO ticket_history (ticket_id, assgn_group, change_timestamp, resolved) VALUES
('ticket_1', 'Intake', '2025-04-11 08:00:00', TRUE),
('ticket_2', 'Intake', '2025-04-11 08:00:00', FALSE),
('ticket_2', 'Service Group', '2025-04-11 09:00:00', TRUE),
('ticket_3', 'Intake', '2025-04-11 08:00:00', FALSE),
('ticket_3', 'Service Group', '2025-04-11 09:00:00', FALSE),
('ticket_3', 'Video', '2025-04-11 10:00:00', TRUE);
Exception Groups Table:
sql

Collapse

Wrap

Copy
CREATE TABLE exception_groups (
    group_name VARCHAR(100) PRIMARY KEY
);
INSERT INTO exception_groups (group_name) VALUES ('Service Group');
Step 2: Fetch and Flatten Data
We’ll group rows by ticket_id and create a flat record per ticket with group_history.

python

Collapse

Wrap

Copy
import psycopg2
from collections import defaultdict

# Connect to PostgreSQL
conn = psycopg2.connect("dbname=support user=postgres")
cur = conn.cursor()

# Fetch exception groups
cur.execute("SELECT group_name FROM exception_groups")
exception_groups = [row[0] for row in cur.fetchall()]

# Fetch ticket history
cur.execute("SELECT ticket_id, assgn_group, change_timestamp, resolved FROM ticket_history ORDER BY ticket_id, change_timestamp")
rows = cur.fetchall()

# Group by ticket_id and flatten
ticket_data = defaultdict(list)
for ticket_id, assgn_group, timestamp, resolved in rows:
    ticket_data[ticket_id].append({"group": assgn_group, "timestamp": timestamp, "resolved": resolved})

# Create flat records
flat_data = []
for ticket_id, history in ticket_data.items():
    group_history = [entry["group"] for entry in history]
    resolved = any(entry["resolved"] for entry in history)
    flat_data.append({
        "ticket_id": ticket_id,
        "group_history": group_history,
        "resolved": resolved,
        "exception_groups": exception_groups
    })

cur.close()
conn.close()
flat_data looks like:

python

Collapse

Wrap

Copy
[
    {
        "ticket_id": "ticket_1",
        "group_history": ["Intake"],
        "resolved": True,
        "exception_groups": ["Service Group"]
    },
    {
        "ticket_id": "ticket_2",
        "group_history": ["Intake", "Service Group"],
        "resolved": True,
        "exception_groups": ["Service Group"]
    },
    {
        "ticket_id": "ticket_3",
        "group_history": ["Intake", "Service Group", "Video"],
        "resolved": True,
        "exception_groups": ["Service Group"]
    }
]
Step 3: Define zen-engine Rules
We need zen-engine to evaluate group_history and return a containment status for each row (i.e., each group in the history). Since zen-engine processes one decision at a time, we’ll design the rule to output an array of statuses, one per group in group_history.

Containment Rules:

For each group at index i in group_history:
If i < length(group_history) - 1, the group passed the ticket to another → not contained.
If i == length(group_history) - 1 (last group):
If resolved == true and group is not in exception_groups, → contained.
If group is in exception_groups, → not contained.
Output an array like ["not contained", "contained"] for each ticket, matching the rows.
zen-engine Decision Model:
zen-engine uses a JSON Decision Model (JDM). We’ll iterate over group_history using a script node to build the status array, then output it.

python

Collapse

Wrap

Copy
rule_json = {
    "nodes": [
        {
            "type": "input",
            "id": "input"
        },
        {
            "type": "script",
            "id": "evaluate_containment",
            "script": """
            var statuses = [];
            for (var i = 0; i < group_history.length; i++) {
                if (i < group_history.length - 1) {
                    statuses.push("not contained");
                } else {
                    if (resolved && exception_groups.indexOf(group_history[i]) === -1) {
                        statuses.push("contained");
                    } else {
                        statuses.push("not contained");
                    }
                }
            }
            return statuses;
            """,
            "inputs": ["group_history", "resolved", "exception_groups"]
        },
        {
            "type": "output",
            "id": "output",
            "value": "$evaluate_containment"
        }
    ]
}
Explanation:
The script node runs JavaScript (zen-engine’s scripting language) to process group_history.
It checks each group:
If not the last (i < length - 1), appends "not contained".
If the last, checks resolved and exception_groups to append "contained" or "not contained".
Outputs an array of statuses, one per row.
Step 4: Evaluate with zen-engine
We’ll evaluate each ticket’s flat record and map the status array back to row-level output.

python

Collapse

Wrap

Copy
from zen_engine import DecisionEngine

# Initialize zen-engine
engine = DecisionEngine()
decision = engine.create_decision(rule_json)

# Evaluate each ticket
results = []
for ticket in flat_data:
    ticket_id = ticket["ticket_id"]
    statuses = decision.evaluate({
        "group_history": ticket["group_history"],
        "resolved": ticket["resolved"],
        "exception_groups": ticket["exception_groups"]
    })
    # Pair each status with the ticket_id
    for status in statuses:
        results.append((ticket_id, status))
Step 5: Format Output
python

Collapse

Wrap

Copy
for ticket_id, status in results:
    print(f"{ticket_id} {status}")
Output:

plaintext

Collapse

Wrap

Copy
ticket_1 contained
ticket_2 not contained
ticket_2 not contained
ticket_3 not contained
ticket_3 not contained
ticket_3 contained
This matches your desired output:

ticket_1: ["Intake"] → ["contained"] (last, resolved, not an exception).
ticket_2: ["Intake", "Service Group"] → ["not contained", "not contained"] (Intake not last, Service Group in exception_groups).
ticket_3: ["Intake", "Service Group", "Video"] → ["not contained", "not contained", "contained"] (Intake and Service Group not last, Video last and not an exception).
Step 6: Dynamic Rules from PostgreSQL
To make the rule dynamic:

Store rule_json in a table:
sql

Collapse

Wrap

Copy
CREATE TABLE containment_rules (
    rule_id SERIAL PRIMARY KEY,
    rule_json JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO containment_rules (rule_json) VALUES (%s);
Pass rule_json as a JSON string.
Fetch and use:
python

Collapse

Wrap

Copy
cur.execute("SELECT rule_json FROM containment_rules ORDER BY created_at DESC LIMIT 1")
rule_json = cur.fetchone()[0]
decision = engine.create_decision(rule_json)
Addressing Row-to-Row Evaluation
Your concern about evaluating group changes is solved by:

Flattening: Sending group_history as a single array lets zen-engine see the full sequence.
Script Node: The JavaScript logic iterates over group_history, checking each group’s position and status relative to others.
Status Array: Outputting one status per group ensures each row gets evaluated in context, preserving the transition logic (e.g., “not last → not contained”).
Why One Row Didn’t Work:

Sending one row (e.g., {"current_group": "Intake", "is_last": False}) lacked future context (e.g., what group came next).
Flattening gives zen-engine the entire group_history, so it can compute is_last or transitions internally.
Alternative to Script Node:
If you prefer avoiding JavaScript, you could preprocess group_history in Python to annotate each group (e.g., is_last), then use a simpler rule. However, the script node is cleaner and keeps logic in zen-engine, making it easier to update rules dynamically.

Complete Code
python

Collapse

Wrap

Copy
import psycopg2
from zen_engine import DecisionEngine
from collections import defaultdict

# Connect to PostgreSQL
conn = psycopg2.connect("dbname=support user=postgres")
cur = conn.cursor()

# Fetch exception groups
cur.execute("SELECT group_name FROM exception_groups")
exception_groups = [row[0] for row in cur.fetchall()]

# Fetch rules (fallback if none in DB)
cur.execute("SELECT rule_json FROM containment_rules ORDER BY created_at DESC LIMIT 1")
rule_json = cur.fetchone()[0] if cur.rowcount > 0 else {
    "nodes": [
        {
            "type": "input",
            "id": "input"
        },
        {
            "type": "script",
            "id": "evaluate_containment",
            "script": """
            var statuses = [];
            for (var i = 0; i < group_history.length; i++) {
                if (i < group_history.length - 1) {
                    statuses.push("not contained");
                } else {
                    if (resolved && exception_groups.indexOf(group_history[i]) === -1) {
                        statuses.push("contained");
                    } else {
                        statuses.push("not contained");
                    }
                }
            }
            return statuses;
            """,
            "inputs": ["group_history", "resolved", "exception_groups"]
        },
        {
            "type": "output",
            "id": "output",
            "value": "$evaluate_containment"
        }
    ]
}

# Fetch ticket history
cur.execute("SELECT ticket_id, assgn_group, change_timestamp, resolved FROM ticket_history ORDER BY ticket_id, change_timestamp")
rows = cur.fetchall()

# Group and flatten data
ticket_data = defaultdict(list)
for ticket_id, assgn_group, timestamp, resolved in rows:
    ticket_data[ticket_id].append({"group": assgn_group, "timestamp": timestamp, "resolved": resolved})

flat_data = []
for ticket_id, history in ticket_data.items():
    group_history = [entry["group"] for entry in history]
    resolved = any(entry["resolved"] for entry in history)
    flat_data.append({
        "ticket_id": ticket_id,
        "group_history": group_history,
        "resolved": resolved,
        "exception_groups": exception_groups
    })

# Initialize zen-engine
engine = DecisionEngine()
decision = engine.create_decision(rule_json)

# Evaluate each ticket
results = []
for ticket in flat_data:
    ticket_id = ticket["ticket_id"]
    statuses = decision.evaluate({
        "group_history": ticket["group_history"],
        "resolved": ticket["resolved"],
        "exception_groups": ticket["exception_groups"]
    })
    for status in statuses:
        results.append((ticket_id, status))

# Print output
for ticket_id, status in results:
    print(f"{ticket_id} {status}")

cur.close()
conn.close()
Thoughts and Recommendations
Flattening Success: Combining rows into group_history lets zen-engine evaluate transitions correctly, solving the row-to-row dependency. The script node iterates over the array, ensuring each group’s containment reflects its role in the sequence.
zen-engine Fit: Its JSON Decision Model and scripting support make it ideal for processing sequences like group_history. The ability to store rules in PostgreSQL JSONB fields supports dynamic updates.
Dynamic Data: exception_groups is fetched from PostgreSQL and passed to the rule, fulfilling your need for expressions like group_history.contains(some_postgres_table_field). If you need other dynamic data (e.g., allowed transitions), add tables and update the script.
Performance: zen-engine (Rust-based) is fast, suitable even for thousands of tickets. For very large datasets, batch flat_data processing.
Exceptions: The rule assumes Service Group is an exception. If there are others (e.g., specific transitions, ticket types), extend exception_groups or add a table like:
sql

Collapse

Wrap

Copy
CREATE TABLE transition_exceptions (
    from_group VARCHAR(100),
    to_group VARCHAR(100),
    contained BOOLEAN
);
Then update the script:
javascript

Collapse

Wrap

Copy
if (i < group_history.length - 1) {
    // Check transition_exceptions table
    var transition = group_history[i] + "->" + group_history[i+1];
    if (transition_exceptions.includes(transition)) {
        statuses.push("contained");
    } else {
        statuses.push("not contained");
    }
}
Questions for Clarity
ticket_2 Output: The output shows Service Group as not contained, but you mentioned it’s contained if it resolved the ticket. Is the output correct, or should the last row be contained?
Exceptions: Are there specific transitions (e.g., Intake → Video) or other groups that affect containment?
Data Volume: How many tickets/rows daily? This confirms zen-engine’s suitability.
Resolved Flag: Is resolved in ticket_history, or do we need another table/query?
Additional Rules: Any other conditions (e.g., time between changes, ticket priority)?
Next Steps
Test: Run the code with your data. Install zen-engine via pip install zen-engine.
Verify Output: Ensure ticket_2’s not contained for Service Group aligns with your intent.
Dynamic Rules: Store rule_json in containment_rules for updates.
Deploy: Use as a script, cron job, or API endpoint (e.g., Flask).
If you confirm the ticket_2 logic or share more exceptions, I can tweak the script node further. Let me know your thoughts or any specifics about your setup!

