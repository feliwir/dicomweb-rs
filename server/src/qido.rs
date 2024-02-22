use actix_web::{get, web, HttpResponse, Responder};
use dicom_json::DicomJson;
use dicom_object::{InMemDicomObject, Tag};
use serde::Deserialize;

use crate::DicomWebServer;

/// QIDO-RS
///
/// See https://www.dicomstandard.org/using/dicomweb/query-qido-rs for more information
/// More detail can be found in PS3.18 10.6.
#[derive(Deserialize, Debug)]
pub struct QidoStudyQuery {
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub fuzzymatching: Option<bool>,
    #[serde(skip_deserializing)]
    pub includefields: Vec<String>,
    #[serde(skip_deserializing)]
    pub matches: Vec<(Tag, String)>,
}

#[get("/studies")]
pub async fn search_studies_all(
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
pub async fn search_series_study_level(
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
    query: web::Query<QidoSeriesQuery>,
) -> impl Responder {
    let result = (callbacks.search_series)(Some(&study_uid), &query);

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

#[get("/studies/{study_uid}/instances")]
pub async fn search_instances_study_level(
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
    query: web::Query<QidoInstanceQuery>,
) -> impl Responder {
    let result = (callbacks.search_instances)(Some(&study_uid), None, &query);

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

#[get("/series")]
pub async fn search_series_all(
    callbacks: web::Data<DicomWebServer>,
    query: web::Query<QidoSeriesQuery>,
) -> impl Responder {
    let result = (callbacks.search_series)(None, &query);

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
pub async fn search_instances_series_level(
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
    series_uid: web::Path<String>,
    query: web::Query<QidoInstanceQuery>,
) -> impl Responder {
    let result = (callbacks.search_instances)(Some(&study_uid), Some(&series_uid), &query);

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

#[get("/instances")]
pub async fn search_instances_all(
    callbacks: web::Data<DicomWebServer>,
    query: web::Query<QidoInstanceQuery>,
) -> impl Responder {
    let result = (callbacks.search_instances)(None, None, &query);

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

pub fn qido_config(cfg: &mut web::ServiceConfig) {
    cfg.service(search_studies_all)
        .service(search_series_all)
        .service(search_series_study_level)
        .service(search_instances_all)
        .service(search_instances_study_level)
        .service(search_instances_series_level);
}
