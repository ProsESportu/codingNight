mod prisma;
mod router;
use base64::prelude::*;
use router::AppState;
use std::sync::Arc;
use tokio::net::TcpListener;
#[tokio::main]
async fn main() {
    let router = router::router();
    let cors = tower_http::cors::CorsLayer::permissive();
    let db = Arc::new(prisma::new_client().await.unwrap());
    let app = axum::Router::new()
        .nest(
            "/rspc",
            rspc_axum::endpoint(router, move |parts: axum::http::request::Parts| {
                let mut session_id = None;

                if let Some(header_value) = parts.headers.get("Authorization") {
                    // println!("{:?}", header_value);
                    if let Ok(e) = header_value.to_str() {
                        if let Some((_, bearer)) = e.split_once(" ") {
                            if let Ok(id) = BASE64_STANDARD.decode(bearer) {
                                if id.len() == 32 {
                                    let mut array = [0u8; 32];
                                    array.copy_from_slice(&id);
                                    session_id = Some(array);
                                };
                            };
                        };
                    };
                };
                AppState { db, session_id }
            }),
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
