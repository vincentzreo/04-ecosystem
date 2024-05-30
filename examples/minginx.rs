use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::net::{TcpListener, TcpStream};
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    listen_addr: String,
    upstream_addr: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = resolve_config();
    let config = Arc::new(config);
    info!("Upstream address: {}", config.upstream_addr);
    info!("Listen address: {}", config.listen_addr);

    let listener = TcpListener::bind(&config.listen_addr).await?;
    loop {
        let (client, addr) = listener.accept().await?;
        let clone_config = config.clone();
        info!("Accepted connection from: {}", addr);
        tokio::spawn(async move {
            let upstream = TcpStream::connect(&clone_config.upstream_addr).await?;
            proxy(client, upstream).await?;
            Ok::<(), anyhow::Error>(())
        });
    }
    #[allow(unreachable_code)]
    Ok(())
}

async fn proxy(mut client: TcpStream, mut upstream: TcpStream) -> anyhow::Result<()> {
    let (mut client_read, mut client_write) = client.split();
    let (mut upstream_read, mut upstream_write) = upstream.split();
    let client_to_upstream = tokio::io::copy(&mut client_read, &mut upstream_write);
    let upstream_to_client = tokio::io::copy(&mut upstream_read, &mut client_write);
    if let Err(e) = tokio::try_join!(client_to_upstream, upstream_to_client) {
        warn!("Error proxying data: {:?}", e);
    };
    Ok(())
}

fn resolve_config() -> Config {
    Config {
        listen_addr: "0.0.0.0:8081".to_string(),
        upstream_addr: "0.0.0.0:8080".to_string(),
    }
}
