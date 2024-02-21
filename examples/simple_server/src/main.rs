use std::{env, fs};

use actix_web::{web, App, HttpServer};
use dicom::{dictionary_std::tags, object::InMemDicomObject};
use dicom_object::FileDicomObject;
use dicomweb_server::{dicomweb_config, QidoInstanceQuery, QidoSeriesQuery, QidoStudyQuery};

fn search_study(
    _query: &QidoStudyQuery,
) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>> {
    unimplemented!()
}

fn search_series(
    _study_uid: &str,
    _query: &QidoSeriesQuery,
) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>> {
    unimplemented!()
}

fn search_instance(
    _study_uid: &str,
    _series_uid: &str,
    _query: &QidoInstanceQuery,
) -> Result<InMemDicomObject, Box<dyn std::error::Error>> {
    unimplemented!()
}

fn store_instances(
    instances: &Vec<FileDicomObject<InMemDicomObject>>,
) -> Result<(), Box<dyn std::error::Error>> {
    print!("");
    for instance in instances {
        let study_uid = instance.element(tags::STUDY_INSTANCE_UID)?.to_str()?;
        let series_uid = instance.element(tags::SERIES_INSTANCE_UID)?.to_str()?;
        let sop_uid = instance.element(tags::SOP_INSTANCE_UID)?.to_str()?;
        fs::create_dir_all(format!("data/{}/{}/", study_uid, series_uid))?;
        instance.write_to_file(format!("data/{}/{}/{}.dcm", study_uid, series_uid, sop_uid))?;
    }
    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug,actix_web=debug");
    env_logger::init();

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(dicomweb_server::DicomWebServer {
                search_instance: search_instance,
                search_series: search_series,
                search_study: search_study,
                store_instances: store_instances,
            }))
            .configure(dicomweb_config)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
