
const POSTGRES_URL: &str = "pgsql://username:password@localhost/database";

use sqlx::postgres::PgPoolOptions;
use sqlx::types::Uuid;

#[derive(Debug, sqlx::FromRow)]
struct User {
    id: Uuid,

    login_name: String,
    password_hash: String,

    display_name: Option<String>
}

async fn test_postgres() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new().connect(POSTGRES_URL).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    sqlx::query("INSERT INTO users (login_name, password_hash) VALUES ($1, $2) ON CONFLICT (login_name) DO NOTHING")
        .bind("memphis")
        .bind("password123!")
        .execute(&pool)
        .await?;

    let users: Vec<User> = sqlx::query_as("SELECT * FROM users")
        .fetch_all(&pool)
        .await?;

    println!("{:?}", users);

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