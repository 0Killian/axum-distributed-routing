use axum::http::StatusCode;
use axum_distributed_routing::create_router;
use axum_distributed_routing::route;
use axum_distributed_routing::route_group;

// Create the root route group
route_group!(Routes, ());

// You can nest groups
route_group!(pub Api, (), Routes, "/api");

route!(
    group = Routes,
    path = "/echo/{str:String}",
    method = GET,
    async test_fn -> String { str }
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
