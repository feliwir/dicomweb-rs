use dicom::object::InMemDicomObject;

mod multipart;
mod qido;
mod stow;
mod tags;
mod wado;

use dicom_object::FileDicomObject;
use qido::*;
use stow::*;
use wado::*;

pub use qido::{qido_config, QidoInstanceQuery, QidoSeriesQuery, QidoStudyQuery};
pub use stow::stow_config;
pub use wado::wado_config;

/// DICOMWeb Server
/// Provide the callbacks for the QIDO-RS and WADO-RS endpoints.
pub struct DicomWebServer {
    pub search_study:
        fn(&QidoStudyQuery) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>>,
    pub search_series:
        fn(&str, &QidoSeriesQuery) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>>,
    pub search_instance:
        fn(&str, &str, &QidoInstanceQuery) -> Result<InMemDicomObject, Box<dyn std::error::Error>>,
    pub store_instances:
        fn(&Vec<FileDicomObject<InMemDicomObject>>) -> Result<(), Box<dyn std::error::Error>>,
}

pub fn dicomweb_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(store_instances)
        .service(store_instances_for_study)
        .service(search_studies)
        .service(search_series)
        .service(search_instances)
        .service(retrieve_instance)
        .service(retrieve_series)
        .service(retrieve_study);
}
