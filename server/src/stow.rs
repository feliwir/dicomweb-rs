
use actix_web::{
    post,
    web::{self}, HttpMessage, HttpRequest, HttpResponse, Responder,
};
use dicom::dictionary_std::tags;
use dicom_json::DicomJson;
use dicom_object::{FileDicomObject, InMemDicomObject};
use futures_util::StreamExt;

use crate::{multipart::MultipartRelated, DicomWebServer};

async fn collect_dicom_files(
    request: HttpRequest,
    mut payload: MultipartRelated,
) -> Result<Vec<FileDicomObject<InMemDicomObject>>, String> {
    let content_type = request.content_type();
    let mut dicom_files = Vec::new();

    // Check if the content type is multipart/related
    if content_type == "multipart/related" {
        // iterate over multipart stream
        while let Some(item) = payload.next().await {
            let mut obj = match item {
                Ok(obj) => obj,
                Err(e) => return Err(e.to_string()),
            };

            let inner_content_type = obj.content_type();
            match inner_content_type {
                Some(inner_content_type) => {
                    if inner_content_type.to_string() == "application/dicom" {
                        let mut data: Vec<u8> = Vec::new();

                        // Merge chunks into one array
                        while let Some(chunk) = obj.next().await {
                            match chunk {
                                Ok(chunk) => data.extend_from_slice(&chunk),
                                Err(e) => return Err(e.to_string()),
                            }
                        }
                        dicom_files.push(FileDicomObject::from_reader(data.as_slice()));
                    }
                }
                None => return Err(String::from("Missing content type")),
            }
        }
    }

    // Filter the failed DICOM files
    let dicom_files = dicom_files
        .into_iter()
        .filter_map(|dcm| match dcm {
            Ok(dcm) => Some(dcm),
            Err(e) => {
                log::error!("Failed to parse DICOM file: {}", e);
                None
            }
        })
        .collect();

    Ok(dicom_files)
}

/// STOW-RS
///
/// See https://www.dicomstandard.org/using/dicomweb/store-stow-rs for more information
#[post("/studies")]
pub async fn store_instances(
    request: HttpRequest,
    payload: MultipartRelated,
    callbacks: web::Data<DicomWebServer>,
    // MultipartForm(form): MultipartForm<InstancesUpload>,
) -> impl Responder {
    // Collect the DICOM files
    let dicom_files = match collect_dicom_files(request, payload).await {
        Ok(dicom_files) => dicom_files,
        Err(e) => return HttpResponse::BadRequest().body(e),
    };

    // Store the files
    let result = (callbacks.store_instances)(&dicom_files);
    if let Err(e) = result {
        return HttpResponse::InternalServerError().body(e.to_string());
    }

    // Transform the DICOM objects into JSON
    let mut dcm_list: Vec<InMemDicomObject> = dicom_files
        .into_iter()
        .map(|dcm| dcm.into_inner())
        .collect();

    // Remove the pixel data from the DICOM objects
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
