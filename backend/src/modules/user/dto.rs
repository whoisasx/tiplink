use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDetails{
  pub email: String,
  pub avatar_url: String,
  pub wallet: String,
  pub name: String
}