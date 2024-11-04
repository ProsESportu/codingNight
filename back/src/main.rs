mod prisma;
mod router;
use std::sync::Arc;

use router::AppState;
use std::sync::Mutex;
use tokio::net::TcpListener;
#[tokio::main]
async fn main() {
    let router = router::router();
    let cors = tower_http::cors::CorsLayer::permissive();
    let db = Arc::new(prisma::new_client().await.unwrap());
    let app = axum::Router::new()
        .nest(
            "/rspc",
            rspc_axum::endpoint(router, move || AppState { db }),
        )
        .layer(cors);
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    // It is highly recommended to unit test your rspc router by creating it
    // This will ensure it doesn't have any issues and also export updated Typescript types.

    use crate::router;

    #[test]
    fn test_rspc_router() {
        router::router();
    }
}
