use std::env;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Config {
    /// Port the MPC HTTP server listens on.
    pub port: u16,
    /// PostgreSQL connection string — shared with the backend database.
    pub database_url: String,
    /// Shared HMAC secret used to verify escrow-signer requests from the backend.
    /// Must match `ESCROW_HMAC_SECRET` in the backend environment.
    pub escrow_hmac_secret: String,
    /// High-entropy master secret used to:
    ///   (a) deterministically derive per-user ed25519 keypairs, and
    ///   (b) encrypt the derived private key before storing it in the database.
    /// Keep this in a secrets manager in production; never commit it.
    pub master_secret: String,
    /// Base58-encoded 64-byte secret key of the platform escrow account.
    /// Used to sign transfers initiated by the escrow signer.
    pub escrow_private_key: String,
    /// Solana JSON-RPC endpoint.
    pub solana_rpc_url: String,
}

impl Config {
    pub fn init() -> Config {
        let port = env::var("MPC_PORT")
            .unwrap_or_else(|_| String::from("8081"))
            .parse::<u16>()
            .expect("MPC_PORT must be a valid port number");
        let database_url        = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let escrow_hmac_secret  = env::var("ESCROW_HMAC_SECRET").expect("ESCROW_HMAC_SECRET must be set");
        let master_secret       = env::var("MPC_MASTER_SECRET").expect("MPC_MASTER_SECRET must be set");
        let escrow_private_key  = env::var("ESCROW_PRIVATE_KEY").expect("ESCROW_PRIVATE_KEY must be set");
        let solana_rpc_url      = env::var("SOLANA_RPC_URL").expect("SOLANA_RPC_URL must be set");

        Config {
            port,
            database_url,
            escrow_hmac_secret,
            master_secret,
            escrow_private_key,
            solana_rpc_url,
        }
    }
}
