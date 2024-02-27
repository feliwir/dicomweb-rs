use actix_web::{get, http, web, HttpRequest, HttpResponse, Responder};
use dicom_json::DicomJson;
use dicom_object::InMemDicomObject;


use crate::{DicomWebServer, QidoInstanceQuery, QidoSeriesQuery, QidoStudyQuery};

#[get("/studies")]
pub async fn search_studies_all(
    request: HttpRequest,
    callbacks: web::Data<DicomWebServer>,
    query: web::Query<QidoStudyQuery>,
) -> impl Responder {
    // See https://dicom.nema.org/medical/dicom/current/output/html/part18.html#sect_8.7.5
    // "The response to a request without an Accept header field shall be 406 (Not Acceptable)"
    let accept_header = request.headers().get(http::header::ACCEPT);
    if accept_header.is_none() {
        return HttpResponse::NotAcceptable().finish();
    }

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

#[get("/studies/{study_uid}/series")]
pub async fn search_series_study_level(
    request: HttpRequest,
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
    query: web::Query<QidoSeriesQuery>,
) -> impl Responder {
    // See https://dicom.nema.org/medical/dicom/current/output/html/part18.html#sect_8.7.5
    // "The response to a request without an Accept header field shall be 406 (Not Acceptable)"
    let accept_header = request.headers().get(http::header::ACCEPT);
    if accept_header.is_none() {
        return HttpResponse::NotAcceptable().finish();
    }

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
    request: HttpRequest,
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
    query: web::Query<QidoInstanceQuery>,
) -> impl Responder {
    // See https://dicom.nema.org/medical/dicom/current/output/html/part18.html#sect_8.7.5
    // "The response to a request without an Accept header field shall be 406 (Not Acceptable)"
    let accept_header = request.headers().get(http::header::ACCEPT);
    if accept_header.is_none() {
        return HttpResponse::NotAcceptable().finish();
    }

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
    request: HttpRequest,
    callbacks: web::Data<DicomWebServer>,
    query: web::Query<QidoSeriesQuery>,
) -> impl Responder {
    // See https://dicom.nema.org/medical/dicom/current/output/html/part18.html#sect_8.7.5
    // "The response to a request without an Accept header field shall be 406 (Not Acceptable)"
    let accept_header = request.headers().get(http::header::ACCEPT);
    if accept_header.is_none() {
        return HttpResponse::NotAcceptable().finish();
    }

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

#[get("/studies/{study_uid}/series/{series_uid}/instances")]
pub async fn search_instances_series_level(
    request: HttpRequest,
    callbacks: web::Data<DicomWebServer>,
    study_uid: web::Path<String>,
    series_uid: web::Path<String>,
    query: web::Query<QidoInstanceQuery>,
) -> impl Responder {
    // See https://dicom.nema.org/medical/dicom/current/output/html/part18.html#sect_8.7.5
    // "The response to a request without an Accept header field shall be 406 (Not Acceptable)"
    let accept_header = request.headers().get(http::header::ACCEPT);
    if accept_header.is_none() {
        return HttpResponse::NotAcceptable().finish();
    }

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
    request: HttpRequest,
    callbacks: web::Data<DicomWebServer>,
    query: web::Query<QidoInstanceQuery>,
) -> impl Responder {
    // See https://dicom.nema.org/medical/dicom/current/output/html/part18.html#sect_8.7.5
    // "The response to a request without an Accept header field shall be 406 (Not Acceptable)"
    let accept_header = request.headers().get(http::header::ACCEPT);
    if accept_header.is_none() {
        return HttpResponse::NotAcceptable().finish();
    }

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
