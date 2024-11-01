use web_server::{create_app_config_from_env, create_router};

#[tokio::main]
async fn main() {
    let config = create_app_config_from_env().await;
    let router = create_router(config);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:31416")
        .await
        .unwrap();

    #[cfg(feature = "debug")]
    let router = {
        let sock_address = listener.local_addr().unwrap();

        tracing_subscriber::fmt()
            .with_max_level(tracing_subscriber::filter::LevelFilter::TRACE)
            .pretty()
            .init();
        log::info!("listening on http://{}", sock_address);

        router.layer(tower_http::trace::TraceLayer::new_for_http())
    };

    axum::serve(listener, router).await.unwrap();
}
