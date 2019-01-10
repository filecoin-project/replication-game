use std::fmt;
use std::io::Cursor;

use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{Responder, Response};
use rocket_contrib::json;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub struct ApiError(failure::Error);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T: Into<failure::Error>> From<T> for ApiError {
    fn from(t: T) -> ApiError {
        ApiError(t.into())
    }
}
impl<'a> Responder<'a> for ApiError {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        println!("Error {}", self);

        // Create JSON response
        let resp = json!({
            "status": "failure",
            "message": format!("Error: {}", self),
        })
        .to_string();

        // Respond. The `Ok` here is a bit of a misnomer. It means we
        // successfully created an error response
        Ok(Response::build()
            .status(Status::BadRequest)
            .header(ContentType::JSON)
            .sized_body(Cursor::new(resp))
            .finalize())
    }
}
