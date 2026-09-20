use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{routing::get, Router, extract::State};

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn status(&self) -> &str {
        "Server is running"
    }
}

type ApiSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

async fn graphql_handler(State(schema): State<ApiSchema>, request: GraphQLRequest) -> GraphQLResponse {
    schema.execute(request.into_inner()).await.into()
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    let application = Router::new()
        .route("/graphql", get(graphql_handler).post(graphql_handler))
        .with_state(schema);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, application).await.unwrap();
}