use surrealism::surrealism;
use surrealdb_types::Value;

#[surrealism]
pub fn hello() -> String {
    "Hello from Rust + Surrealism! 🚀".to_string()
}

#[surrealism]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}


#[surrealism]
pub fn fetch_and_store_users() -> Result<String, String> {
    // The API is fetched via SurrealQL's built-in http::get function inside the DB engine,
    // which prevents multi-threaded WASM compilation issues and fulfills the single-threaded constraint.
    let query = "INSERT INTO user (SELECT * FROM http::get('https://fake-json-api.mock.beeceptor.com/users'));";
    
    // Execute the SQL query from the WASM runtime
    match surrealism::imports::sql::<_, Value>(query) {
        Ok(_) => Ok("Successfully fetched and inserted users.".to_string()),
        Err(e) => Err(format!("Failed to execute query: {}", e)),
    }
}


#[surrealism]
pub fn fetch_and_store_airflow_health() -> Result<String, String> {
    // The API is fetched via SurrealQL's built-in http::get function inside the DB engine.
    // We add a checked_at timestamp and convert the heartbeat strings into native datetime objects.
    let query = r#"
        INSERT INTO airflow_health (
            SELECT 
                time::now() AS checked_at,
                metadatabase,
                { 
                    status: scheduler.status, 
                    latest_scheduler_heartbeat: type::datetime(scheduler.latest_scheduler_heartbeat) 
                } AS scheduler,
                { 
                    status: triggerer.status, 
                    latest_triggerer_heartbeat: type::datetime(triggerer.latest_triggerer_heartbeat) 
                } AS triggerer,
                { 
                    status: dag_processor.status, 
                    latest_dag_processor_heartbeat: type::datetime(dag_processor.latest_dag_processor_heartbeat) 
                } AS dag_processor
            FROM http::get('http://host.containers.internal:8080/api/v2/monitor/health', { 'Authorization': 'Basic YWRtaW46WUZnZVdyR2NkMnU4cFlCRw==' })
        );
    "#;
    
    // Execute the SQL query from the WASM runtime
    match surrealism::imports::sql::<_, Value>(query) {
        Ok(_) => Ok("Successfully fetched and inserted Airflow health status.".to_string()),
        Err(e) => Err(format!("Failed to execute query: {}", e)),
    }
}