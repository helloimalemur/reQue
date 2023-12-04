use rocket::http::Status;
use rocket::outcome::Outcome;
use rocket::request::{FromRequest, Request};

pub struct ApiKey<'r>(&'r str);

#[derive(Debug)]
pub enum ApiKeyError {
    MissingError,
    InvalidError,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey<'r> {
    type Error = ApiKeyError;

    async fn from_request(
        req: &'r Request<'_>,
    ) -> Outcome<ApiKey<'r>, (Status, ApiKeyError), Status> {
        let _e = ApiKeyError::InvalidError;
        let _e2 = ApiKeyError::MissingError;

        match req.headers().get_one("x-api-key") {
            Some(key) => Outcome::Success(ApiKey(key)),
            None => Outcome::Success(ApiKey("")),
        }
    }
}

impl<'r> ToString for ApiKey<'r> {
    fn to_string(&self) -> String {
        String::from_utf8(Vec::from(self.0.as_bytes())).unwrap()
    }
}
