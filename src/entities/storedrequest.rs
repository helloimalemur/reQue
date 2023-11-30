#[derive(Debug, Clone)]
pub struct StoredRequest {
    pub method: String,
    pub host: String,
    pub port: u16,
    pub uri: String,
    pub headers: Vec<String>,
    pub body: String
}
