use actix_multipart::form::{tempfile::TempFile, MultipartForm};
use actix_web::{post, web, HttpResponse, Responder};
use dicom_json::DicomJson;
use dicom_object::InMemDicomObject;

use crate::DicomWebServer;

/// STOW-RS
///
/// See https://www.dicomstandard.org/using/dicomweb/store-stow-rs for more information
#[derive(Debug, MultipartForm)]
pub struct InstancesUpload {
    #[multipart(rename = "file")]
    files: Vec<TempFile>,
}

#[post("/studies")]
pub async fn store_instances(
    callbacks: web::Data<DicomWebServer>,
    MultipartForm(form): MultipartForm<InstancesUpload>,
) -> impl Responder {
    let mut dcm_file_list = Vec::new();
    for file in form.files {
        // Open DICOM file
        let result = dicom::object::open_file(file.file);
        match result {
            Ok(dcm_file) => dcm_file_list.push(dcm_file),
            Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
        }
    }

    // Store the files
    let result = (callbacks.store_instances)(&dcm_file_list);
    if let Err(e) = result {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    // Transform the DICOM objects into JSON
    let dcm_list: Vec<InMemDicomObject> = dcm_file_list
        .into_iter()
        .map(|dcm| dcm.into_inner())
        .collect();

    for dcm in &mut dcm_list {
        dcm.remove_element(tags::PIXEL_DATA);
    }

    let dcm_json = DicomJson::from(dcm_list);
    HttpResponse::Ok().json(dcm_json)
}

#[post("/studies/{study_uid}")]
pub async fn store_instances_for_study(_study_uid: web::Path<String>) -> impl Responder {
    HttpResponse::Ok().body("store_instances")
}

pub fn stow_config(cfg: &mut web::ServiceConfig) {
    cfg.service(store_instances)
        .service(store_instances_for_study);
}
