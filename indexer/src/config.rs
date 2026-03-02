use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub solana_rpc_url: String,
}

impl Config {
    pub fn init() -> Config {
        let host = env::var("INDEXER_HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
        let port = env::var("INDEXER_PORT")
            .unwrap_or_else(|_| String::from("8080"))
            .parse::<u16>()
            .expect("INDEXER_PORT must be a valid port number");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let solana_rpc_url = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set");

        Config {
            host,
            port,
            database_url,
            solana_rpc_url,
        }
    }
}
