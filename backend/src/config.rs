use std::env;

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct Config{
  pub host: String,
  pub port: u16,
  pub client_origin: String,
  pub jwt_secret: String,
  pub jwt_expires_in: String,
  pub jwt_max_age: i64,
  pub google_oauth_client_id: String,
  pub google_oauth_client_secret: String,
  pub redirect_url: String,
  pub database_url: String,
  pub csrf_state_token: String,
  pub mpc_server_url: String,
  pub mpc_secret: String,
  pub escrow_public_key: String,
  pub escrow_hmac_secret: String,
  pub base_url: String,
  pub jupiter_quote_url: String,
  pub jupiter_swap_url: String,
  pub sol_mint: String,
  pub solana_base_fee: u64,
}

impl Config{
  pub fn init() -> Config{
    let host = env::var("BACKEND_HOST").unwrap_or_else(|_| String::from("0.0.0.0"));
    let port = env::var("BACKEND_PORT")
      .unwrap_or_else(|_| String::from("3000"))
      .parse::<u16>()
      .expect("BACKEND_PORT must be a valid port number");
    let client_origin = env::var("CLIENT_ORIGIN").expect("CLIENT_ORIGIN must be set");
    let jwt_secret=env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_expires_in=env::var("TOKEN_EXPIRED_IN").expect("TOKEN_EXPIRED_IN must be set");
    let jwt_max_age=env::var("TOKEN_MAXAGE").expect("TOKEN_MAXAGE must be set");
    let google_oauth_client_id= env::var("GOOGLE_OAUTH_CLIENT_ID").expect("GOOGLE_OAUTH_CLIENT_ID must be set");
    let google_oauth_client_secret=env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("GOOGLE_OAUTH_CLIENT_SECRET must be set");
    let redirect_url=env::var("REDIRECT_URL").expect("REDIRECT_URL must be set");
    let database_url=env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let csrf_state_token = env::var("RANDOM_CSRF_STRING").expect("RANDOM_CSRF_STRING must be set");
    let mpc_server_url = env::var("MPC_SERVER_URL").expect("MPC_SERVER_URL must be set");
    let mpc_secret = env::var("MPC_SECRET").expect("MPC_SECRET must be set");
    let escrow_public_key = env::var("ESCROW_PUBLIC_KEY").expect("ESCROW_PUBLIC_KEY must be set");
    let escrow_hmac_secret = env::var("ESCROW_HMAC_SECRET").expect("ESCROW_HMAC_SECRET must be set");
    let base_url = env::var("BASE_URL").expect("BASE_URL must be set");
    let jupiter_quote_url = env::var("JUPITER_QUOTE_URL").expect("JUPITER_QUOTE_URL must be set");
    let jupiter_swap_url  = env::var("JUPITER_SWAP_URL").expect("JUPITER_SWAP_URL must be set");
    let sol_mint          = env::var("SOL_MINT").expect("SOL_MINT must be set");
    let solana_base_fee   = env::var("SOLANA_BASE_FEE_LAMPORTS")
        .unwrap_or_else(|_| String::from("5000"))
        .parse::<u64>()
        .unwrap_or(5_000);

    Config {
      host,
      port,
      client_origin,
      jwt_secret,
      jwt_expires_in,
      jwt_max_age: jwt_max_age.parse::<i64>().unwrap_or(60 * 24 * 3600),
      google_oauth_client_id,
      google_oauth_client_secret,
      redirect_url,
      database_url,
      csrf_state_token,
      mpc_server_url,
      mpc_secret,
      escrow_public_key,
      escrow_hmac_secret,
      base_url,
      jupiter_quote_url,
      jupiter_swap_url,
      sol_mint,
      solana_base_fee,
    }
  }
}