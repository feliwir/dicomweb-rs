//! MultipartRelated payload support

use actix_utils::future::{ready, Ready};
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};

use crate::multipart::MultipartReader;

impl FromRequest for MultipartReader {
    type Error = Error;
    type Future = Ready<Result<MultipartReader, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        ready(Ok(match MultipartReader::boundary(req.headers()) {
            Ok(boundary) => MultipartReader::from_boundary(boundary, payload.take()),
            Err(err) => MultipartReader::from_error(err),
        }))
    }
}
