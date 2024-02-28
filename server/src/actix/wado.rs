use std::{io::Write};

use actix_web::{get, web, HttpResponse, Responder};
use dicom::dictionary_std::tags;
use dicom_json::DicomJson;
use dicom_object::InMemDicomObject;
use dicom_pixeldata::PixelDecoder;

use crate::{actix::MultipartWriter, DicomWebServer};

/// WADO-RS
///
///
#[get("/studies/{study_uid}")]
pub async fn retrieve_study(
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
) -> impl Responder {
    let result = (callbacks.retrieve_study)(&study_uid);

    match result {
        Ok(dcm_files) => {
            let mut mp = MultipartWriter::new();
            for dcm_file in dcm_files {
                let mut data: Vec<u8> = Vec::new();

                // Write the DICOM file to memory and add it to our stream
                if let Err(e) = dcm_file.write_all(&mut data) {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }

                if let Err(e) = mp.add(&*data, "Content-Type: application/dicom") {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }
            }

            // Finish the multipart stream
            mp.finish();

            let content_type = format!(
                "multipart/related; type=application/dicom; boundary={}",
                mp.boundary
            );

            return HttpResponse::Ok().content_type(content_type).body(mp.data);
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/studies/{study_uid}/metadata")]
pub async fn retrieve_study_metadata(
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
) -> impl Responder {
    let result = (callbacks.retrieve_study)(&study_uid);

    match result {
        Ok(dcm_files) => {
            // Apply the offset and the filter
            let mut filtered: Vec<InMemDicomObject> = dcm_files
                .into_iter()
                .map(|dcm_file| dcm_file.into_inner())
                .collect();

            // Remove any bulk data
            for dcm in &mut filtered {
                dcm.remove_element(tags::PIXEL_DATA);
            }

            return HttpResponse::Ok().json(DicomJson::from(filtered));
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/studies/{study_uid}/series/{series_uid}")]
pub async fn retrieve_series(
    callbacks: web::Data<DicomWebServer>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (study_uid, series_uid) = path.into_inner();
    let result = (callbacks.retrieve_series)(&study_uid, &series_uid);

    match result {
        Ok(dcm_files) => {
            let mut mp = MultipartWriter::new();
            for dcm_file in dcm_files {
                let mut data: Vec<u8> = Vec::new();

                // Write the DICOM file to memory and add it to our stream
                if let Err(e) = dcm_file.write_all(&mut data) {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }

                if let Err(e) = mp.add(&*data, "Content-Type: application/dicom") {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }
            }

            // Finish the multipart stream
            mp.finish();

            let content_type = format!(
                "multipart/related; type=application/dicom; boundary={}",
                mp.boundary
            );

            return HttpResponse::Ok().content_type(content_type).body(mp.data);
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/studies/{study_uid}/series/{series_uid}/metadata")]
pub async fn retrieve_series_metadata(
    callbacks: web::Data<DicomWebServer>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (study_uid, series_uid) = path.into_inner();
    let result = (callbacks.retrieve_series)(&study_uid, &series_uid);

    match result {
        Ok(dcm_files) => {
            // Apply the offset and the filter
            let mut filtered: Vec<InMemDicomObject> = dcm_files
                .into_iter()
                .map(|dcm_file| dcm_file.into_inner())
                .collect();

            // Remove any bulk data
            for dcm in &mut filtered {
                dcm.remove_element(tags::PIXEL_DATA);
            }

            return HttpResponse::Ok().json(DicomJson::from(filtered));
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
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

            // Finish the multipart stream
            mp.finish();

            let content_type = format!(
                "multipart/related; type=application/dicom; boundary={}",
                mp.boundary
            );

            return HttpResponse::Ok().content_type(content_type).body(mp.data);
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/studies/{study_uid}/series/{series_uid}/instances/{instance_uid}/metadata")]
pub async fn retrieve_instance_metadata(
    callbacks: web::Data<DicomWebServer>,
    path: web::Path<(String, String, String)>,
) -> impl Responder {
    let (study_uid, series_uid, instance_uid) = path.into_inner();
    let result = (callbacks.retrieve_instance)(&study_uid, &series_uid, &instance_uid);

    match result {
        Ok(mut dcm_file) => {
            // Remove any bulkdata from the DICOM file
            dcm_file.remove_element(tags::PIXEL_DATA);

            return HttpResponse::Ok().json(DicomJson::from(dcm_file));
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/studies/{study_uid}/series/{series_uid}/instances/{instance_uid}/frames/{frame_list}")]
pub async fn retrieve_instance_frames(
    callbacks: web::Data<DicomWebServer>,
    path: web::Path<(String, String, String, String)>,
) -> impl Responder {
    let (study_uid, series_uid, instance_uid, _frame_list) = path.into_inner();
    let result = (callbacks.retrieve_instance)(&study_uid, &series_uid, &instance_uid);

    match result {
        Ok(dcm_file) => {
            // TODO: use the framelist to extract the frames from the pixel data
            if let Ok(pixel_data) = dcm_file.decode_pixel_data() {
                let mut mp = MultipartWriter::new();
                let mut data: Vec<u8> = Vec::new();

                // Write the pixel data to memory and add it to our stream
                if let Err(e) = data.write_all(&pixel_data.data()) {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }

                if let Err(e) = mp.add(&*data, "Content-Type: application/octet-stream") {
                    return HttpResponse::InternalServerError().body(e.to_string());
                }

                // Finish the multipart stream
                mp.finish();

                let content_type = format!(
                    "multipart/related; type=application/octet-stream; boundary={}",
                    mp.boundary
                );

                return HttpResponse::Ok().content_type(content_type).body(mp.data);
            } else {
                return HttpResponse::InternalServerError().body("No pixel data found");
            }
        }
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    }
}

pub fn wado_config(cfg: &mut web::ServiceConfig) {
    cfg.service(retrieve_study)
        .service(retrieve_study_metadata)
        .service(retrieve_series)
        .service(retrieve_series_metadata)
        .service(retrieve_instance)
        .service(retrieve_instance_frames)
        .service(retrieve_instance_metadata);
}
