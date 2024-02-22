//! MultipartRelated payload support

use actix_utils::future::{ready, Ready};
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};

use crate::multipart::MultipartRelated;

impl FromRequest for MultipartRelated {
    type Error = Error;
    type Future = Ready<Result<MultipartRelated, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        ready(Ok(match MultipartRelated::boundary(req.headers()) {
            Ok(boundary) => MultipartRelated::from_boundary(boundary, payload.take()),
            Err(err) => MultipartRelated::from_error(err),
        }))
    }
}
