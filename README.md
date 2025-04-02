# Axum Distributed Routing

> [!WARNING]
> This crate is experimental, use at your own risk!

Provides utilities for generating statically typed distributed routes for axum.

## Usage

```rust
use axum_distributed_routing::*;

struct MyState(String);

route_group!(MyRoutes, MyState);

route!(
    group = MyRoutes,
    path = "/hello/{name:String}",
    method = GET,
    async hello -> String {
        format!("{} {}!", state.0, name)
    }
);

#[tokio::main]
async fn main() {
    let app = create_router!(MyRoutes);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
```
