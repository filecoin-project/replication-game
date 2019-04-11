use std::io::prelude::*;
use std::io::{self, Cursor};

use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rocket::data::{FromData, Outcome, Transform, Transformed};
use rocket::http::Status;
use rocket::{fairing, http, Data, Outcome::*, Request, Response};
use rocket_contrib::json::{Json, JsonError};
use serde::de::Deserialize;

#[derive(Debug)]
pub struct GzipFairing;

impl fairing::Fairing for GzipFairing {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Gzip compression",
            kind: fairing::Kind::Response,
        }
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        let headers = request.headers();

        if headers
            .get("Accept-Encoding")
            .any(|e| e.to_lowercase().contains("gzip"))
        {
            response.body_bytes().and_then(|body| {
                let mut gz = GzEncoder::new(Vec::new(), Compression::default());
                gz.write_all(&body)
                    .and_then(|_| gz.finish())
                    .and_then(|buf| {
                        response.set_sized_body(Cursor::new(buf));
                        response.set_raw_header("Content-Encoding", "gzip");
                        Ok(())
                    })
                    .map_err(|e| eprintln!("{}", e))
                    .ok()
            });
        }

        // Enable cache control on static assets
        let uri = request.uri().path();

        if request.method() == http::Method::Get && !uri.starts_with("/api") && uri != "/" {
            response.set_raw_header("Cache-Control", "public, max-age=31536000");
        } else {
            response.set_raw_header("Cache-Control", "no-cache, no-store, must-revalidate");
        }
    }
}

#[derive(Debug)]
pub struct Gzip<T>(pub T);

impl<T> Gzip<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[derive(Debug)]
pub enum GzipError<'a> {
    Io(io::Error),
    JsonError(JsonError<'a>),
}

const LIMIT: u64 = 1 << 20;

impl<'a, T: Deserialize<'a>> FromData<'a> for Gzip<Json<T>> {
    type Error = GzipError<'a>;
    type Owned = String;
    type Borrowed = str;

    fn transform(r: &Request, data: Data) -> Transform<Outcome<Self::Owned, Self::Error>> {
        let headers = r.headers();

        let is_gzip = headers
            .get("Content-Encoding")
            .any(|e| e.to_lowercase().contains("gzip"));

        if !is_gzip {
            return Transform::Borrowed(Forward(data));
        }

        let size_limit = r.limits().get("json").unwrap_or(LIMIT);
        let mut gz = GzDecoder::new(data.open().take(size_limit));
        let mut s = String::with_capacity(512);

        match gz.read_to_string(&mut s) {
            Ok(_) => Transform::Borrowed(Success(s)),
            Err(e) => Transform::Borrowed(Failure((Status::BadRequest, GzipError::Io(e)))),
        }
    }

    fn from_data(_r: &Request, o: Transformed<'a, Self>) -> Outcome<Self, Self::Error> {
        let string = o.borrowed()?;
        match serde_json::from_str(string) {
            Ok(v) => Success(Gzip(Json(v))),
            Err(e) => {
                if e.is_data() {
                    Failure((
                        Status::UnprocessableEntity,
                        GzipError::JsonError(JsonError::Parse(string, e)),
                    ))
                } else {
                    Failure((
                        Status::BadRequest,
                        GzipError::JsonError(JsonError::Parse(string, e)),
                    ))
                }
            }
        }
    }
}
