use flate2::write::GzEncoder;
use flate2::Compression;
use rocket::{fairing, http, Request, Response};

use std::io::prelude::*;
use std::io::Cursor;

pub struct Gzip;

impl fairing::Fairing for Gzip {
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
