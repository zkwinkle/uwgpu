use web_server::{create_app_config_from_env, create_router};

#[tokio::main]
async fn main() {
    let config = create_app_config_from_env();
    let router = create_router(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:31416")
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}
