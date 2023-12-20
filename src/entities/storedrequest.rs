use rocket::http::HeaderMap;

#[derive(Debug, Clone)]
pub struct StoredRequest<'a> {
    pub method: String,
    pub host: String,
    pub port: u16,
    pub uri: String,
    pub headers: HeaderMap<'a>,
    pub body: String,
}
