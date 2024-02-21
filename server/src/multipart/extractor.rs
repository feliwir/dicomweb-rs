//! MultipartRelated payload support

use actix_utils::future::{ready, Ready};
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};

use crate::multipart::MultipartRelated;

/// Get request's payload as multipart stream.
///
/// Content-type: multipart/form-data;
///
/// # Examples
/// ```
/// use actix_web::{web, HttpResponse, Error};
/// use actix_multipart::MultipartRelated;
/// use futures_util::StreamExt as _;
///
/// async fn index(mut payload: MultipartRelated) -> Result<HttpResponse, Error> {
///     // iterate over multipart stream
///     while let Some(item) = payload.next().await {
///            let mut field = item?;
///
///            // Field in turn is stream of *Bytes* object
///            while let Some(chunk) = field.next().await {
///                println!("-- CHUNK: \n{:?}", std::str::from_utf8(&chunk?));
///            }
///     }
///
///     Ok(HttpResponse::Ok().into())
/// }
/// ```
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
