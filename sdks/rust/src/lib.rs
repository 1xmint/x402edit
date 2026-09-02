pub struct Client {
    base_url: String,
    http: reqwest::Client,
}
impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            http: reqwest::Client::new(),
        }
    }
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }
}
