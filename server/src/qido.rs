use actix_web::{get, web, HttpResponse, Responder};
use dicom_json::DicomJson;
use dicom_object::InMemDicomObject;
use serde::Deserialize;

use crate::DicomWebServer;

/// QIDO-RS
///
/// See https://www.dicomstandard.org/using/dicomweb/query-qido-rs for more information
/// More detail can be found in PS3.18 10.6.
#[derive(Deserialize, Debug)]
pub struct QidoStudyQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    includefield: Option<String>,
    modality: Option<String>,
    patient_name: Option<String>,
    patient_id: Option<String>,
    accession_number: Option<String>,
    study_date: Option<String>,
    study_time: Option<String>,
    study_description: Option<String>,
    referring_physician_name: Option<String>,
    patient_age: Option<String>,
}

#[get("/studies")]
pub async fn search_studies(
    callbacks: web::Data<DicomWebServer>,
    query: web::Query<QidoStudyQuery>,
) -> impl Responder {
    let result = (callbacks.search_study)(&query);

    match result {
        Ok(dcm_list) => {
            // Apply the offset and the filter
            let filtered: Vec<InMemDicomObject> = dcm_list
                .iter()
                .skip(query.offset.unwrap_or(0))
                .take(query.limit.unwrap_or(100))
                .cloned()
                .collect();

            // Convert the results to JSON
            let dcm_json = DicomJson::from(filtered);
            HttpResponse::Ok().json(dcm_json)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize, Debug)]
pub struct QidoSeriesQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    includefield: Option<String>,
    modality: Option<String>,
    series_instance_uid: Option<String>,
    series_description: Option<String>,
}

#[get("/studies/{study_uid}/series")]
pub async fn search_series(
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
    query: web::Query<QidoSeriesQuery>,
) -> impl Responder {
    let result = (callbacks.search_series)(&study_uid, &query);

    match result {
        Ok(dcm_list) => {
            // Apply the offset and the filter
            let filtered: Vec<InMemDicomObject> = dcm_list
                .iter()
                .skip(query.offset.unwrap_or(0))
                .take(query.limit.unwrap_or(100))
                .cloned()
                .collect();

            // Convert the results to JSON
            let dcm_json = DicomJson::from(filtered);
            HttpResponse::Ok().json(dcm_json)
        }
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[derive(Deserialize, Debug)]
pub struct QidoInstanceQuery {
    limit: Option<usize>,
    offset: Option<usize>,
    includefield: Option<String>,
    sop_instance_uid: Option<String>,
    instance_number: Option<String>,
}

#[get("/studies/{study_uid}/series/{series_uid}/instances")]
pub async fn search_instances(
    _study_uid: web::Path<String>,
    _series_uid: web::Path<String>,
    _query: web::Query<QidoInstanceQuery>,
) -> impl Responder {
    HttpResponse::Ok().body("search_instances")
}

pub fn qido_config(cfg: &mut web::ServiceConfig) {
    cfg.service(search_studies)
        .service(search_series)
        .service(search_instances);
}
