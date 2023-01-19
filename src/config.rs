use config::{Config, ConfigError};
use serde::Deserialize;
use std::net::SocketAddr;

/// Struct for parsing config sections.
/// http - common parameters for http server/client.
/// preprocessing - parameters for ms-images-preprocessing.
/// loki - parameters for loki client.
/// redis - parameters for redis client.
/// image_validator - rules for image validator.
#[derive(Debug, Default, Deserialize, Clone)]
pub struct Settings {
    pub http: HTTPServerParameters,
    pub clickhouse: ClickhouseParameters,
    pub prometheus: PrometheusParameters,
}

/// Struct for parsing config section http server/client.
/// host - ip for http server.
/// port - port for http server.
#[derive(Debug, Default, Deserialize, Clone)]
pub struct HTTPServerParameters {
    pub host: Option<String>,
    pub port: Option<u16>,
}

/// Struct for parsing config section clickhouse server.
/// host - host for clickhouse server.
/// port - port for clickhouse server.
/// user - user for clickhouse server.
/// database - database for clickhouse server.
/// queries - list of queries for clickhouse server.
#[derive(Debug, Default, Deserialize, Clone)]
pub struct ClickhouseParameters {
    pub host: Option<String>,
    pub port: Option<String>,
    pub user: Option<String>,
    pub database: Option<String>,
    pub queries: Option<Vec<String>>,
}

/// Struct for parsing config section prometheus.
/// prefix - prefix of metric.
#[derive(Debug, Default, Deserialize, Clone)]
pub struct PrometheusParameters {
    pub prefix: Option<String>,
}

// Trait for init socket for http server.
pub trait HttpServer {
    fn init_http_server_socket(&self) -> SocketAddr;
}

impl Settings {
    /// Init config file.
    ///
    /// This method will panic if the parameters are not fully specified or if don't exist path
    /// to config file (ex. -c ./config/config.toml)
    ///
    /// # Panics
    ///
    /// ```rust,should_panic
    /// static ref CONFIG = Settings::new(&Args::parse().config).unwrap();
    /// ```
    ///
    pub(crate) fn new(config_path: &str) -> Result<Self, ConfigError> {
        let config = Config::builder()
            .add_source(config::File::with_name(config_path))
            .build();
        config?.try_deserialize()
    }
}

impl HttpServer for Settings {
    /// Init http server socket.
    /// If host is not set, then use localhost.
    /// If port is not set, then use default port (8080).
    ///
    /// # Panics
    /// If http server can not be initialized.
    /// 1. Incorrectly formatted config file or not set config properties.
    /// 2. Socket can't be bind.
    ///
    /// # Examples
    /// ```sh
    ///  error creating server listener: Address already in use (os error 48)
    /// ```
    fn init_http_server_socket(&self) -> SocketAddr {
        let host = match self.http.host.as_ref() {
            Some(host) => host.clone(),
            None => "127.0.0.1".to_string(),
        };
        let port = self.http.port.unwrap_or(8080);
        SocketAddr::new(host.parse().unwrap(), port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::ConfigError;

    #[test]
    pub fn test_config_init() -> Result<(), ConfigError> {
        let settings = Settings::new("config/config.toml")?;
        assert_eq!(settings.image_validator.max_height, 5315);
        Ok(())
    }
}
