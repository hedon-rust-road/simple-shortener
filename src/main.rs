use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const SERVER_ADDR: &str = "0.0.0.0:9090";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let listener = TcpListener::bind(SERVER_ADDR).await?;
    let app = Router::new().route("/", get(|| async move { "Hello, World!" }));

    axum::serve(listener, app).await?;
    Ok(())
}
