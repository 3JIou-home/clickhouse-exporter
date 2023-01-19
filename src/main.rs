mod config;

use std::net::SocketAddr;
use axum::{Extension, Router};
use axum::extract::State;
use axum::routing::get;
use clap::builder::Str;
use serde::{Deserialize, Serialize};
use clickhouse::{error::Result, sql, Client, Row};
use clap::Parser;
use lazy_static::lazy_static;
use crate::config::{HttpServer, Settings};
use log::error;

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
    #[clap(short, long)]
    config: String,
}

lazy_static! {
    /// Init config file from argument command line (-c).
    ///
    /// # Examples
    ///
    /// ```rust
    /// let config = Settings::new("./config/config.toml").expect("config can be loaded");
    /// assert_eq!(config.check_parameters.timeout_between_checks, Some(10));
    /// ```

    static ref CONFIG: Settings = Settings::new(&Args::parse().config).expect("config file can be loaded");
}

#[derive(Debug, Serialize, Deserialize, Row)]
struct Event {
    metric: String,
    value: i64,
}

async fn get_metrics(
    Extension(client): Extension<Client>,
    Extension(clickhouse_queries): Extension<Vec<String>>,
    Extension(prometheus_prefix): Extension<String>,
) -> String {
    let mut system_metrics = Vec::new();
    let mut result_vec = Vec::new();
    for query in clickhouse_queries {
        system_metrics = client.query(query.as_str())
            .fetch_all::<Event>()
            .await
            .unwrap();
        for event in system_metrics.iter() {
            result_vec.push(format!("{}: {}\n", format!("{:?}.system.metrics.{}:", prometheus_prefix, event.metric.to_lowercase()), event.value));
        }
    };
    result_vec.join("")
}

#[tokio::main]
async fn main() {
    let clickhouse_database = match CONFIG.clickhouse.database {
        Some(ref database) => database.to_string(),
        None => "default".to_string(),
    };
    let clickhouse_user = match CONFIG.clickhouse.user {
        Some(ref user) => user.to_string(),
        None => "default".to_string(),
    };
    let clickhouse_queries = match CONFIG.clickhouse.queries {
        Some(ref queries) => queries.to_vec(),
        None => vec!["SELECT 1".to_string()],
    };
    let prometheus_prefix = match CONFIG.prometheus.prefix {
        Some(ref prefix) => prefix.to_string(),
        None => "default".to_string(),
    };
    let clickhouse_host = match CONFIG.clickhouse.host {
        Some(ref host) => host.to_string(),
        None => "localhost".to_string(),
    };
    let clickhouse_port = match CONFIG.clickhouse.port {
        Some(ref port) => port.to_string(),
        None => "8123".to_string(),
    };
    let client = Client::default()
        .with_url(format!("http://{}:{}", clickhouse_host, clickhouse_port))
        .with_database(clickhouse_database)
        .with_user(clickhouse_user)
        .with_compression(clickhouse::Compression::Lz4);
    // Init socket from config file.
    let server: SocketAddr = CONFIG.init_http_server_socket();
    // Init router.
    let app = Router::new().route("/metrics", get(get_metrics))
        .layer(Extension(client))
        .layer(Extension(clickhouse_queries))
        .layer(Extension(prometheus_prefix));
    // Start server.
    axum::Server::bind(&server)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
