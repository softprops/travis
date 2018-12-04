//! interfaces for interacting with travis commit descriptions

#[derive(Debug, Deserialize, Clone)]
pub struct Commit {
    pub id: usize,
    // standard rep fields
    pub sha: String,
    #[serde(rename="ref")]
    pub git_ref: String,
    pub message: String,
    pub compare_url: String,
    pub committed_at: String,
}
