use async_graphql::{Context, EmptySubscription, Object, Schema, SimpleObject, InputObject};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::post, Router};
use sqlx::{postgres::PgPoolOptions, PgPool, types::Uuid, FromRow};
use tokio::net::TcpListener;
use scylla::client::session_builder::SessionBuilder;
use scylla_migrate::Migrator;
use scylla::client::session::Session;

#[derive(Debug, FromRow, SimpleObject)]
struct User {
    id: Uuid,

    username: String,
    password_hash: String,

    display_name: Option<String>
}

#[derive(Debug, FromRow, SimpleObject)]
struct Channel {
    id: Uuid,
    id_category: Option<Uuid>,

    class: String,
    name: String
}

#[derive(InputObject)]
struct CreateChannelInput {
    id_category: Option<Uuid>,
    class: String,
    name: String
}

#[derive(InputObject)]
struct UpdateChannelInput {
    id: Uuid,
    id_category: Option<Uuid>,
    name: String
}

struct Mutation;

#[Object]
impl Mutation {
    async fn create_channel(&self, context: &Context<'_>, input: CreateChannelInput) -> async_graphql::Result<Channel> {
        let pool = context.data::<PgPool>()?;

        let query = "INSERT INTO channels (id_category, class, name) VALUES ($1, $2, $3) RETURNING *";
        let channel = sqlx::query_as::<_, Channel>(query)
            .bind(input.id_category)
            .bind(input.class)
            .bind(input.name)
            .fetch_one(pool)
            .await?;

        Ok(channel)
    }

    async fn update_channel(&self, context: &Context<'_>, input: UpdateChannelInput) -> async_graphql::Result<Channel> {
        let pool = context.data::<PgPool>()?;

        let query = "UPDATE channels SET id_category = COALESCE($1, id_category), name = COALESCE($2, name) WHERE id = $3 RETURNING *";
        let channel = sqlx::query_as::<_, Channel>(query)
            .bind(input.id_category)
            .bind(input.name)
            .bind(input.id)
            .fetch_one(pool)
            .await?;

        Ok(channel)
    }

    async fn delete_channel(&self, context: &Context<'_>, id: Uuid) -> async_graphql::Result<bool> {
        let pool = context.data::<PgPool>()?;

        let query = "DELETE FROM channels WHERE id = $1";
        let result = sqlx::query(query)
            .bind(id)
            .execute(pool)
            .await?;

        let success = result.rows_affected() > 0;
        Ok(success)
    }
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

    async fn channel(&self, context: &Context<'_>, id: Uuid) -> async_graphql::Result<Option<Channel>> {
        let pool = context.data::<PgPool>()?;

        let channel = sqlx::query_as::<_, Channel>("SELECT * FROM channels WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(channel)
    }

    async fn channels(&self, context: &Context<'_>) -> async_graphql::Result<Vec<Channel>> {
        let pool = context.data::<PgPool>()?;

        let channels = sqlx::query_as::<_, Channel>("SELECT * FROM channels")
            .fetch_all(pool)
            .await?;

        Ok(channels)
    }
}

type ApplicationSchema = Schema<Query, Mutation, EmptySubscription>;

const POSTGRES_URL: &str = "pgsql://username:password@localhost/database";
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

const CASSANDRA_URL : &str = "localhost:9042";
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

    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(postgres)
        .finish();

    let application = Router::new()
        .route("/graphql", post(graphql_handler))
        .with_state(schema);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, application).await?;

    Ok(())
}