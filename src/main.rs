use async_graphql::{Context, EmptyMutation, EmptySubscription, Object, Schema, SimpleObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::post, Router};
use sqlx::{postgres::PgPoolOptions, PgPool, types::Uuid, FromRow};
use tokio::net::TcpListener;
use scylla::client::session_builder::SessionBuilder;
use scylla_migrate::Migrator;
use scylla::client::session::Session;

const POSTGRES_URL: &str = "pgsql://username:password@localhost/database";
const CASSANDRA_URL : &str = "localhost:9042";

#[derive(Debug, FromRow, SimpleObject)]
struct User {
    id: Uuid,

    username: String,
    password_hash: String,

    display_name: Option<String>
}

struct Query;

#[Object]
impl Query {
    async fn user(&self, context: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<User>> {
        let pool = context.data::<PgPool>()?;

        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }
}

type ApplicationSchema = Schema<Query, EmptyMutation, EmptySubscription>;

async fn graphql_handler(
    schema: axum::extract::State<ApplicationSchema>,
    request: GraphQLRequest
) -> GraphQLResponse {
    schema.execute(request.into_inner()).await.into()
}

async fn setup_postgres() -> Result<PgPool, Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .connect(POSTGRES_URL)
        .await?;

    sqlx::migrate!("./migrations/postgres").run(&pool).await?;

    Ok(pool)
}

async fn setup_cassandra() -> Result<Session, Box<dyn std::error::Error>> {
    let session = SessionBuilder::new()
        .known_node(CASSANDRA_URL)
        .build()
        .await?;

    let cassandra_migrator = Migrator::new(&session, "./migrations/cassandra");
    cassandra_migrator.run().await?;

    Ok(session)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let postgres = setup_postgres().await?;
    let cassandra = setup_cassandra().await?;

    let schema = Schema::build(Query, EmptyMutation, EmptySubscription)
        .data(postgres)
        .finish();

    let application = Router::new()
        .route("/graphql", post(graphql_handler))
        .with_state(schema);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, application).await?;

    Ok(())
}