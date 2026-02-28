use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateLinkRequest {
    pub amount:    u64,
    pub mint:      Option<String>,
    pub note:      Option<String>,
    /// ISO-8601 datetime string, e.g. "2026-03-01T00:00:00Z"
    pub expiry_at: Option<String>,
}

#[derive(Deserialize)]
pub struct ClaimLinkRequest {
    pub claimer_wallet: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LinkStatus {
    Active,
    Claimed,
    Cancelled,
    Expired,
    All,
}

impl Default for LinkStatus {
    fn default() -> Self {
        LinkStatus::All
    }
}

fn default_page()  -> u32 { 1 }
fn default_limit() -> u32 { 20 }

#[derive(Deserialize)]
pub struct MyLinksQuery {
    #[serde(default = "default_page")]
    pub page:   u32,
    #[serde(default = "default_limit")]
    pub limit:  u32,
    #[serde(default)]
    pub status: LinkStatus,
}

#[derive(Serialize)]
pub struct CreateLinkResponse {
    pub link_id:       String,
    pub link_token:    String,
    pub link_url:      String,
    pub escrow_pubkey: String,
    pub amount:        u64,
    pub mint:          Option<String>,
    pub note:          Option<String>,
    pub expiry_at:     Option<String>,
    pub status:        LinkStatus,
    pub created_at:    String,
}

#[derive(Serialize)]
pub struct LinkPagination {
    pub page:        u32,
    pub limit:       u32,
    pub total:       u64,
    pub total_pages: u64,
    pub has_next:    bool,
    pub has_prev:    bool,
}

#[derive(Serialize)]
pub struct LinkListItem {
    pub link_id:        String,
    pub link_url:       String,
    pub amount:         u64,
    pub mint:           Option<String>,
    pub note:           Option<String>,
    pub status:         LinkStatus,
    pub expiry_at:      Option<String>,
    pub claimer_wallet: Option<String>,
    pub claimed_at:     Option<String>,
    pub created_at:     String,
}

#[derive(Serialize)]
pub struct MyLinksResponse {
    pub pagination: LinkPagination,
    pub links:      Vec<LinkListItem>,
}

#[derive(Serialize)]
pub struct LinkInfoResponse {
    pub link_id:    String,
    pub amount:     u64,
    pub mint:       Option<String>,
    pub symbol:     String,
    pub note:       Option<String>,
    pub status:     LinkStatus,
    pub expiry_at:  Option<String>,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct ClaimLinkResponse {
    pub signature:      String,
    pub amount:         u64,
    pub mint:           Option<String>,
    pub claimer_wallet: String,
    pub claimed_at:     String,
}

#[derive(Serialize)]
pub struct CancelLinkResponse {
    pub signature: String,
    pub status:    LinkStatus,
}