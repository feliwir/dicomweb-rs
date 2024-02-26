use dicom::{dictionary_std::tags, object::InMemDicomObject};

mod filter;
mod multipart;
mod qido;
mod stow;
mod wado;

use dicom_object::{FileDicomObject, Tag};
use qido::*;
use stow::*;
use wado::*;

pub use filter::study_filter;
pub use qido::{qido_config, QidoInstanceQuery, QidoSeriesQuery, QidoStudyQuery};
pub use stow::stow_config;
pub use wado::wado_config;

/// DICOMWeb Server
/// Provide the callbacks for the QIDO-RS and WADO-RS endpoints.
pub struct DicomWebServer {
    pub search_study:
        fn(&QidoStudyQuery) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>>,
    pub search_series: fn(
        Option<&str>, // study_instance_uid
        &QidoSeriesQuery,
    ) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>>,
    pub search_instances: fn(
        Option<&str>, // study_instance_uid
        Option<&str>, // series_instance_uid
        &QidoInstanceQuery,
    ) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>>,
    pub retrieve_study:
        fn(
            &str, // study_instance_uid
        ) -> Result<Vec<FileDicomObject<InMemDicomObject>>, Box<dyn std::error::Error>>,
    pub retrieve_series:
        fn(
            &str, // study_instance_uid
            &str, // series_instance_uid
        ) -> Result<Vec<FileDicomObject<InMemDicomObject>>, Box<dyn std::error::Error>>,
    pub retrieve_instance:
        fn(
            &str, // study_instance_uid
            &str, // series_instance_uid
            &str, // sop_instance_uid
        ) -> Result<FileDicomObject<InMemDicomObject>, Box<dyn std::error::Error>>,
    pub store_instances:
        fn(&Vec<FileDicomObject<InMemDicomObject>>) -> Result<(), Box<dyn std::error::Error>>,
}

// http://dicom.nema.org/medical/dicom/current/output/chtml/part18/sect_10.6.html#table_10.6.1-5
pub const STUDY_TAGS: [Tag; 9] = [
    tags::STUDY_DATE,
    tags::STUDY_TIME,
    tags::ACCESSION_NUMBER,
    tags::MODALITIES_IN_STUDY,
    tags::REFERRING_PHYSICIAN_NAME,
    tags::PATIENT_NAME,
    tags::PATIENT_ID,
    tags::STUDY_INSTANCE_UID,
    tags::STUDY_ID,
];

pub const SERIES_TAGS: [Tag; 6] = [
    tags::MODALITY,
    tags::SERIES_INSTANCE_UID,
    tags::SERIES_NUMBER,
    tags::PERFORMED_PROCEDURE_STEP_START_DATE,
    tags::PERFORMED_PROCEDURE_STEP_START_TIME,
    tags::REQUEST_ATTRIBUTES_SEQUENCE,
];

pub const INSTANCE_TAGS: [Tag; 3] = [
    tags::SOP_CLASS_UID,
    tags::SOP_INSTANCE_UID,
    tags::INSTANCE_NUMBER,
];

pub fn dicomweb_config(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(store_instances)
        .service(store_instances_for_study)
        .service(search_studies_all)
        .service(search_series_all)
        .service(search_series_study_level)
        .service(search_instances_all)
        .service(search_instances_study_level)
        .service(search_instances_series_level)
        .service(retrieve_instance)
        .service(retrieve_series)
        .service(retrieve_study);
}
