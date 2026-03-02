use std::env;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub mpc_secret: String,
    pub escrow_hmac_secret: String,
    pub master_secret: String,
    pub escrow_private_key: String,
    pub solana_rpc_url: String,
}

impl Config {
    pub fn init() -> Config {
        let host = env::var("MPC_HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
        let port = env::var("MPC_PORT")
            .unwrap_or_else(|_| String::from("4000"))
            .parse::<u16>()
            .expect("MPC_PORT must be a valid port number");
        let database_url        = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let mpc_secret          = env::var("MPC_SECRET").expect("MPC_SECRET must be set");
        let escrow_hmac_secret  = env::var("ESCROW_HMAC_SECRET").expect("ESCROW_HMAC_SECRET must be set");
        let master_secret       = env::var("MPC_MASTER_SECRET").expect("MPC_MASTER_SECRET must be set");
        let escrow_private_key  = env::var("ESCROW_PRIVATE_KEY").expect("ESCROW_PRIVATE_KEY must be set");
        let solana_rpc_url      = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set");

        Config {
            host,
            port,
            database_url,
            mpc_secret,
            escrow_hmac_secret,
            master_secret,
            escrow_private_key,
            solana_rpc_url,
        }
    }
}
