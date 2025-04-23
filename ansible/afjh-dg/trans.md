```
import psycopg2

# Database connection parameters
db_params = {
    'dbname': 'your_database',
    'user': 'your_username',
    'password': 'your_password',
    'host': 'localhost',
    'port': '5432'
}

try:
    # Connect to PostgreSQL and create cursor
    conn = psycopg2.connect(**db_params)
    cursor = conn.cursor()
    print("Database connection and cursor created successfully")
except Exception as e:
    print(f"Error connecting to database: {e}")
    

```

```
import uuid

def transfer_to_target(source_table):
    """
    Reads data from a source table and writes to a fixed target table with generated UUIDs.
    
    Args:
        source_table (str): Name of the source table to read from
    """
    try:
        # Read from source table
        cursor.execute(f"SELECT column1, column2 FROM {source_table}")
        records = cursor.fetchall()
        
        # Write to target table (fixed name: target_table)
        for record in records:
            # Generate UUID for ID field
            new_id = str(uuid.uuid4())
            
            # Insert into target table
            cursor.execute(
                """
                INSERT INTO target_table (id, column1, column2)
                VALUES (%s, %s, %s)
                """,
                (new_id, record[0], record[1])
            )
        
        # Commit the transaction
        conn.commit()
        print(f"Successfully transferred {len(records)} records from {source_table} to target_table")
    
    except Exception as e:
        print(f"Error during transfer: {e}")
        conn.rollback()
    
    # Note: Connection and cursor are not closed here to allow reuse in notebook
    # Close them manually when done with: cursor.close(), conn.close()

# Example usage
# transfer_to_target('my_source_table')
```