use axum::http::StatusCode;
use axum::Json;
use axum_distributed_routing::create_router;
use axum_distributed_routing::route;
use axum_distributed_routing::route_group;
use serde::Deserialize;

// Create the root route group
route_group!(Routes, ());

// You can nest groups
route_group!(pub Api, (), Routes, "/api");

#[derive(Deserialize)]
pub struct ExprQuery {
    pub times: i32,
}

#[derive(Deserialize)]
pub struct ExprBody {
    pub plus: i32,
}

// Create a route
route!(
    group = Routes,
    method = GET,

    // You can define path parameters...
    path = "/expr/{val:i32}",

    // ...query parameters...
    query = ExprQuery,

    // ...and body parameters.
    body = Json<ExprBody>,

    // You can also add attributes to the handler
    #[axum::debug_handler]
    async test_fn -> String {
        format!(
            "{} * {} + {} = {}",
            val,
            query.times,
            body.plus,
            val * query.times + body.plus
        )
    }
);

route!(
    group = Api,
    path = "/health",
    method = GET,
    async api_health -> (StatusCode, &'static str) { (axum::http::StatusCode::OK, "ok") }
);

#[tokio::main]
async fn main() {
    // Create the router by calling `create_router!` with the root group
    let router = create_router!(Routes);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    // You can access a route individually.
    println!("{:?}", ROUTE_API_HEALTH);

    axum::serve(listener, router).await.unwrap();
}
