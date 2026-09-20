
const POSTGRES_URL: &str = "pgsql://username:password@localhost/database";

use sqlx::postgres::PgPoolOptions;
async fn test_postgres() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(POSTGRES_URL)
        .await?;

    sqlx::query("CREATE TABLE IF NOT EXISTS users (
        id INTEGER PRIMARY KEY,
        username VARCHAR(32) UNIQUE,
        password VARCHAR(32)
    )")
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO users (id, username, password) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING")
        .bind(2_i32)
        .bind("memphis33333")
        .bind("password123!")
        .execute(&pool)
        .await?;

    let row: Vec<(i32, String, String)> = sqlx::query_as("SELECT * FROM users")
        .fetch_all(&pool)
        .await?;

    println!("{:?}", row);

    Ok(())
}

use scylla::client::session_builder::SessionBuilder;
async fn test_cassandra() -> Result<(), Box<dyn std::error::Error>>{
    let session = SessionBuilder::new()
        .known_node("localhost:9042")
        .build()
        .await?;

    session.query_unpaged(
        "CREATE KEYSPACE IF NOT EXISTS history WITH REPLICATION = {'class' : 'SimpleStrategy', 'replication_factor' : 1}",
        (),
    ).await?;

    session.query_unpaged(
        "CREATE TABLE IF NOT EXISTS history.messages (
            channel_id INT PRIMARY KEY,
            content TEXT
        )",
        (),
    ).await?;

    session.query_unpaged(
        "INSERT INTO history.messages (channel_id, content) VALUES (?, ?)",
        (2_i32, "Hello World!")
    ).await?;

    let result = session.query_unpaged(
        "SELECT content FROM history.messages",
        ()
    ).await?.into_rows_result()?;

    for row in result.rows()? {
        let (content,): (String,) = row?;
        println!("{}", content);
    }

    Ok(())
}

// async fn test_valkey() -> Result<(), sqlx::Error>{
//
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_postgres().await?;
    test_cassandra().await?;
    // test_valkey().await?;
    
    Ok(())
}