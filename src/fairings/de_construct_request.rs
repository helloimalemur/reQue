use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{FromRequest, Request};

#[derive(Debug)]
pub struct RRequest {
    pub method: String,
    pub host: String,
    pub port: u16,
    pub uri: String,
    pub headers: Vec<String>,
    pub body: String,
}

#[derive(Debug)]
pub enum RRequestError {
    MissingError,
    InvalidError,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RRequest {
    type Error = RRequestError;

    async fn from_request(
        req: &'r Request<'_>,
    ) -> Outcome<RRequest, (Status, RRequestError), Status> {
        let rr = Outcome::Success(RRequest {
            method: req.method().to_string(),
            host: req.host().unwrap().to_string(),
            port: 0,
            uri: req.uri().to_string(),
            headers: vec![],
            body: "".to_string(),
        });
        rr
    }
}
