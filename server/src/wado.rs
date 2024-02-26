use std::io::Cursor;

use actix_web::{get, web, HttpResponse, Responder};

use crate::{multipart::MultipartWriter, DicomWebServer};

/// WADO-RS
///
///
#[get("/studies/{study_uid}")]
pub async fn retrieve_study(_study_uid: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body("retrieve_study")
}

#[get("/studies/{study_uid}/series/{series_uid}")]
pub async fn retrieve_series(
    _study_uid: web::Path<String>,
    _series_uid: web::Path<String>,
) -> impl Responder {
    HttpResponse::Ok().body("retrieve_series")
}

#[get("/studies/{study_uid}/series/{series_uid}/instances/{instance_uid}")]
pub async fn retrieve_instance(
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
    series_uid: web::Path<String>,
    instance_uid: web::Path<String>,
) -> impl Responder {
    let result = (callbacks.retrieve_instance)(&study_uid, &series_uid, &instance_uid);

    match result {
        Ok(dcm_file) => {
            let mut mp = MultipartWriter::new();

            // Write the DICOM file to memory and add it to our stream
            let mut cursor = Cursor::new(Vec::new());
            if let Err(e) = dcm_file.write_all(cursor.clone()) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }

            if let Err(e) = mp.add(&mut cursor) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }

            return HttpResponse::Ok()
                .content_type("multipart/related; type=application/dicom")
                .body(mp.data);
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn wado_config(cfg: &mut web::ServiceConfig) {
    cfg.service(retrieve_study)
        .service(retrieve_series)
        .service(retrieve_instance);
}
