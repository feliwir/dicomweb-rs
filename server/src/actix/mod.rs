mod extractor;
mod multipart;
mod qido;
mod stow;
mod wado;

use multipart::*;
use qido::*;
use stow::*;
use wado::*;

pub use qido::qido_config;
pub use stow::stow_config;
pub use wado::wado_config;

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
