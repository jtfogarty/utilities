I want to add a feature to my surrealism module that returns a health check from Apache Airflow.  Airflow is running at http://localhost:8080 and the username and password is below.

{"username": "admin", "password": "YFgeWrGcd2u8pYBG"}

The health check endpoint is http://localhost:8080/health

This data should be stored in SurrealDB.

I plan to schedule this function to run every 5 minutes via a cron job.

