use rocket::http::{HeaderMap, Status};
use rocket::outcome::Outcome;
use rocket::request::{FromRequest, Request};

#[derive(Debug)]
pub struct RRequest<'a> {
    pub method: String,
    pub host: String,
    pub port: u16,
    pub uri: String,
    pub headers: HeaderMap<'a>,
    pub body: String,
}

#[derive(Debug)]
pub enum RRequestError {
    MissingError,
    InvalidError,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RRequest<'r> {
    type Error = RRequestError;

    async fn from_request(
        req: &'r Request<'_>,
    ) -> Outcome<RRequest<'r>, (Status, RRequestError), Status> {
        let _e = RRequestError::InvalidError;
        let _e2 = RRequestError::MissingError;

        let rr = Outcome::Success(RRequest {
            method: req.method().to_string(),
            host: req.host().unwrap().to_string(),
            port: 0,
            uri: req.uri().to_string(),
            headers: req.headers().clone(),
            body: "".to_string(),
        });
        rr
    }
}
