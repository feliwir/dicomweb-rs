use std::{
    fs,
    io::{Cursor, Write},
};

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
    path: web::Path<(String, String, String)>,
) -> impl Responder {
    let (study_uid, series_uid, instance_uid) = path.into_inner();
    let result = (callbacks.retrieve_instance)(&study_uid, &series_uid, &instance_uid);

    match result {
        Ok(dcm_file) => {
            let mut mp = MultipartWriter::new();
            let mut data: Vec<u8> = Vec::new();

            // Write the DICOM file to memory and add it to our stream
            if let Err(e) = dcm_file.write_all(&mut data) {
                return HttpResponse::InternalServerError().body(e.to_string());
            }

            if let Err(e) = mp.add(&*data, "Content-Type: application/dicom") {
                return HttpResponse::InternalServerError().body(e.to_string());
            }
            {
                let mut file = fs::OpenOptions::new()
                    .create(true) // To create a new file
                    .write(true)
                    // either use the ? operator or unwrap since it returns a Result
                    .open("test.txt")
                    .unwrap();

                file.write_all(&mp.data);
            }

            let content_type = format!(
                "multipart/related; type=application/dicom; boundary={}",
                mp.boundary
            );

            return HttpResponse::Ok().content_type(content_type).body(mp.data);
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn wado_config(cfg: &mut web::ServiceConfig) {
    cfg.service(retrieve_study)
        .service(retrieve_series)
        .service(retrieve_instance);
}
