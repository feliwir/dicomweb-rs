use actix_web::{get, web, HttpResponse, Responder};

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
    _study_uid: web::Path<String>,
    _series_uid: web::Path<String>,
    _instance_uid: web::Path<String>,
) -> impl Responder {
    HttpResponse::Ok().body("retrieve_instance")
}

pub fn wado_config(cfg: &mut web::ServiceConfig) {
    cfg.service(retrieve_study)
        .service(retrieve_series)
        .service(retrieve_instance);
}
