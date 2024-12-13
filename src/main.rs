#![deny(unsafe_code, unused_imports)]
#![deny(clippy::all)]

use std::net::TcpListener;

use anyhow::Context;
use tracing::info;
use url_shortener::state::State;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "dotenv")]
    {
        dotenv::dotenv().ok();
    }
    match std::env::var("RUST_LOG").as_deref() {
        Ok("") | Err(_) => std::env::set_var("RUST_LOG", "info"),
        _ => {}
    }

    init_tracing()?;

    let address = {
        let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port: u16 = std::env::var("PORT")
            .context("Missing PORT environment variable")?
            .parse()
            .context("Invalid PORT environment variable")?;
        format!("{host}:{port}")
    };
    info!("Starting API on {address}");

    let database = {
        let connection_str =
            std::env::var("DB_CONNECTION").context("Missing DB_CONNECTION environment variable")?;
        let config: tokio_postgres::Config =
            connection_str.parse().context("Invalid DB_CONNECTION environment variable")?;
        let mgr = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);
        deadpool_postgres::Pool::builder(mgr)
            .runtime(deadpool_postgres::Runtime::Tokio1)
            .build()
            .context("Create database pool")?
    };
    let state = State::new(database);

    let listener = TcpListener::bind(address)?;
    url_shortener::api::listen(listener, state)?.await?;
    Ok(())
}

fn init_tracing() -> anyhow::Result<()> {
    use tracing_log::LogTracer;
    use tracing_subscriber::fmt::time::UtcTime;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::{EnvFilter, Registry};

    LogTracer::init().context("set logger")?;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = tracing_subscriber::fmt::Layer::new()
        .pretty()
        .with_timer(UtcTime::new(time::format_description::well_known::iso8601::Iso8601::DEFAULT))
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true);
    let subscriber = Registry::default().with(env_filter).with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber)
        .context("set tracing default subscriber")?;
    Ok(())
}
