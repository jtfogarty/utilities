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