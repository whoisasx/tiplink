use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateLinkRequest{
  pub amount: u64,
  pub mint: Option<String>,
  pub note: Option<String>,
  pub expiry_at: Option<String>
}

#[derive(Serialize,Deserialize)]
pub enum LinkStatus{
  Acitve,
  Claimed,
  Cancelled,
  Expired,
  All
}

#[derive(Serialize)]
pub struct CreateLinkResponse{
  pub link_id: String,
  pub link_url:  String,
  pub escrow_pubkey: String,
  pub amount: u64,
  pub mint: Option<String>,
  pub note: Option<String>,
  pub expiry_at: Option<String>,
  pub status: LinkStatus,
  pub created_at: String
}

#[derive(Deserialize)]
pub struct MyLinksQuery{
  pub page: u32,
  pub limit: u32,
  pub status: LinkStatus
}

#[derive(Serialize)]
pub struct LinkPaginated{
  pub page: u32,
  pub limit: u32,
  pub total: u32,
  pub total_page: u32,
  pub has_next: bool,
  pub has_prev: bool
}

#[derive(Serialize)]
pub struct LinkListItem{
  pub link_id: String,
  pub link_url: String,
  pub amount: u64,
  pub mint: Option<String>,
  pub note: Option<String>,
  pub status: LinkStatus,
  pub expiry_at: Option<String>,
  pub claimer_wallet: Option<String>,
  pub claimed_at: Option<String>,
  pub created_at: String
}

#[derive(Serialize)]
pub struct MyLinksResponse{
  pub pagination: LinkPaginated,
  pub links: Vec<LinkListItem>
}

#[derive(Serialize)]
pub struct LinkInfoResponse{
  pub link_id: String,
  pub amount: u64,
  pub mint: Option<String>,
  pub symbol: String,
  pub note: Option<String>,
  pub status: LinkStatus,
  pub expiry_at: Option<String>,
  pub created_at: String
}

#[derive(Deserialize)]
pub struct ClaimLinkRequest{
  claimer_wallet: String
}

#[derive(Serialize)]
pub struct ClaimLinkResponse{
  signature: String,
  amount: String,
  mint: Option<String>,
  claimer_wallet: String,
  claimed_at: String
}

#[derive(Serialize)]
pub struct CancelLinkResponse{
  signature: String,
  status: LinkStatus
}