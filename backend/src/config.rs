use std::env;

#[allow(dead_code)]
#[derive(Debug,Clone)]
pub struct Config{
  pub client_origin: String,
  pub jwt_secret: String,
  pub jwt_expires_in: String,
  pub jwt_max_age: i64,
  pub google_oauth_client_id: String,
  pub google_oauth_client_secret: String,
  pub redirect_url: String,
  pub database_url: String,
  pub random_string: String
}

impl Config{
  pub fn init() -> Config{
    let client_origin = env::var("CLIENT_ORIGIN").expect("CLIENT_ORIGIN must be set");
    let jwt_secret=env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let jwt_expires_in=env::var("TOKEN_EXPIRED_IN").expect("TOKEN_EXPIRED_IN must be set");
    let jwt_max_age=env::var("TOKEN_MAXAGE").expect("TOKEN_MAXAGE must be set");
    let google_oauth_client_id= env::var("GOOGLE_OAUTH_CLIENT_ID").expect("GOOGLE_OAUTH_CLIENT_ID must be set");
    let google_oauth_client_secret=env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("GOOGLE_OAUTH_CLIENT_SECRET must be set");
    let redirect_url=env::var("REDIRECT_URL").expect("REDIRECT_URL must be set");
    let database_url=env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let random_string=env::var("RANDOM_CSRF_STRING").expect("RANDOM_CSRF_STRING must be set");

    Config{
      client_origin,
      jwt_secret,
      jwt_expires_in,
      jwt_max_age: jwt_max_age.parse::<i64>().unwrap_or(60 * 24 * 3600),
      google_oauth_client_id,
      google_oauth_client_secret,
      redirect_url,
      database_url,
      random_string
    }
  }
}