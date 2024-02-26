use std::{env, fs};

use actix_web::{middleware, web, App, HttpServer};
use dicom::{
    core::{DataElement, PrimitiveValue},
    dictionary_std::tags,
    object::InMemDicomObject,
};
use dicom_object::FileDicomObject;
use dicomweb_server::{
    dicomweb_config, study_filter, QidoInstanceQuery, QidoSeriesQuery, QidoStudyQuery,
    INSTANCE_TAGS, SERIES_TAGS, STUDY_TAGS,
};
use itertools::Itertools;
use walkdir::WalkDir;

const DATA_DIR: &str = "data";
const SELF_URL: &str = "127.0.0.1:8080";

fn get_all_data_files() -> Vec<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(DATA_DIR) {
        let entry = entry.unwrap();
        if entry.file_type().is_file() {
            files.push(entry.path().to_str().unwrap().to_string());
        }
    }
    files
}

fn search_study(
    query: &QidoStudyQuery,
) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>> {
    // Collect all files in the data directory
    let files = get_all_data_files();
    let mut dcm_files = Vec::new();
    for file in files {
        dcm_files.push(FileDicomObject::open_file(&file)?.into_inner());
    }

    let studys = dcm_files
        .iter()
        // Apply the filter parameters
        .filter(|dcm| study_filter(dcm, query))
        // Only keep one instance per study
        .unique_by(|dcm| dcm.get(tags::STUDY_INSTANCE_UID).unwrap().to_str().unwrap())
        // Only keep the study tags
        .map(|dcm| {
            let mut study = InMemDicomObject::from_element_iter(
                dcm.clone()
                    .into_iter()
                    .filter(|elt| STUDY_TAGS.contains(&elt.header().tag))
                    .map(|elt| elt.clone()),
            );

            // Add the retrieve URL
            let url = format!(
                "http://{}/studies/{}",
                SELF_URL,
                dcm.element(tags::STUDY_INSTANCE_UID)
                    .unwrap()
                    .to_str()
                    .unwrap()
            );
            study.put(DataElement::new(
                tags::RETRIEVE_URL,
                dicom::core::VR::UR,
                PrimitiveValue::from(url),
            ));

            study
        })
        .collect();

    Ok(studys)
}

fn is_series_filtered(_dcm: &InMemDicomObject, _query: &QidoSeriesQuery) -> bool {
    // TODO:
    false
}

fn search_series(
    study_uid: Option<&str>,
    query: &QidoSeriesQuery,
) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>> {
    // Collect all files in the data directory
    let files = get_all_data_files();
    let mut dcm_files = Vec::new();
    for file in files {
        dcm_files.push(FileDicomObject::open_file(&file)?.into_inner());
    }

    let series = dcm_files
        .iter()
        // Only keep the instances of the study
        .filter(|dcm| {
            if study_uid.is_none() {
                return true;
            }

            dcm.element(tags::STUDY_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                == study_uid.unwrap()
        })
        // Only keep one instance per series
        .unique_by(|dcm| {
            dcm.get(tags::SERIES_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
        })
        // Check if the study is filtered
        .filter(|dcm| !is_series_filtered(dcm, query))
        // Only keep the study tags
        .map(|dcm| {
            let mut study = InMemDicomObject::from_element_iter(
                dcm.clone()
                    .into_iter()
                    .filter(|elt| SERIES_TAGS.contains(&elt.header().tag))
                    .map(|elt| elt.clone()),
            );

            // Add the retrieve URL
            let url = format!(
                "http://{}/studies/{}/series/{}",
                SELF_URL,
                dcm.element(tags::STUDY_INSTANCE_UID)
                    .unwrap()
                    .to_str()
                    .unwrap(),
                dcm.element(tags::SERIES_INSTANCE_UID)
                    .unwrap()
                    .to_str()
                    .unwrap()
            );
            study.put(DataElement::new(
                tags::RETRIEVE_URL,
                dicom::core::VR::UR,
                PrimitiveValue::from(url),
            ));

            study
        })
        .collect();

    Ok(series)
}

fn is_instance_filtered(_dcm: &InMemDicomObject, _query: &QidoInstanceQuery) -> bool {
    // TODO
    false
}

fn search_instances(
    study_uid: Option<&str>,
    series_uid: Option<&str>,
    query: &QidoInstanceQuery,
) -> Result<Vec<InMemDicomObject>, Box<dyn std::error::Error>> {
    // Collect all files in the data directory
    let files = get_all_data_files();
    let mut dcm_files = Vec::new();
    for file in files {
        dcm_files.push(FileDicomObject::open_file(&file)?.into_inner());
    }

    let instances = dcm_files
        .iter()
        // Only keep the instances of the study
        .filter(|dcm| {
            if study_uid.is_none() {
                return true;
            }

            dcm.element(tags::STUDY_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                == study_uid.unwrap()
        })
        // Only keep the instances of the series
        .filter(|dcm| {
            if series_uid.is_none() {
                return true;
            }

            dcm.element(tags::SERIES_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                == series_uid.unwrap()
        })
        // This should already be the case - it should not be possible to have multiple files with the same SOP Instance UID
        .unique_by(|dcm| dcm.get(tags::SOP_INSTANCE_UID).unwrap().to_str().unwrap())
        // Check if the study is filtered
        .filter(|dcm| !is_instance_filtered(dcm, query))
        // Only keep the study tags
        .map(|dcm| {
            let mut study = InMemDicomObject::from_element_iter(
                dcm.clone()
                    .into_iter()
                    .filter(|elt| INSTANCE_TAGS.contains(&elt.header().tag))
                    .map(|elt| elt.clone()),
            );

            // Add the retrieve URL
            let url = format!(
                "http://{}/studies/{}/series/{}/instances/{}",
                SELF_URL,
                dcm.element(tags::STUDY_INSTANCE_UID)
                    .unwrap()
                    .to_str()
                    .unwrap(),
                dcm.element(tags::SERIES_INSTANCE_UID)
                    .unwrap()
                    .to_str()
                    .unwrap(),
                dcm.element(tags::SOP_INSTANCE_UID)
                    .unwrap()
                    .to_str()
                    .unwrap()
            );
            study.put(DataElement::new(
                tags::RETRIEVE_URL,
                dicom::core::VR::UR,
                PrimitiveValue::from(url),
            ));

            study
        })
        .collect();

    Ok(instances)
}

fn retrieve_study(
    study_uid: &str,
) -> Result<Vec<FileDicomObject<InMemDicomObject>>, Box<dyn std::error::Error>> {
    let files = get_all_data_files();
    let dcm_files = files
        .iter()
        .filter(|file| {
            let dcm = FileDicomObject::open_file(file).unwrap().into_inner();
            dcm.element(tags::STUDY_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                == study_uid
        })
        .map(|file| FileDicomObject::open_file(file))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(dcm_files)
}

fn retrieve_series(
    study_uid: &str,
    series_uid: &str,
) -> Result<Vec<FileDicomObject<InMemDicomObject>>, Box<dyn std::error::Error>> {
    let files = get_all_data_files();
    let dcm_files = files
        .iter()
        .filter(|file| {
            let dcm = FileDicomObject::open_file(file).unwrap().into_inner();
            if dcm
                .element(tags::STUDY_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                != study_uid
            {
                return false;
            }

            dcm.element(tags::SERIES_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                == series_uid
        })
        .map(|file| FileDicomObject::open_file(file))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(dcm_files)
}

fn retrieve_instance(
    study_uid: &str,
    series_uid: &str,
    sop_instance_uid: &str,
) -> Result<FileDicomObject<InMemDicomObject>, Box<dyn std::error::Error>> {
    let files = get_all_data_files();
    let dcm_files = files
        .iter()
        .filter(|file| {
            let dcm = FileDicomObject::open_file(file).unwrap().into_inner();
            if dcm
                .element(tags::STUDY_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                != study_uid
            {
                return false;
            }

            if dcm
                .element(tags::SERIES_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                != series_uid
            {
                return false;
            }

            dcm.element(tags::SOP_INSTANCE_UID)
                .unwrap()
                .to_str()
                .unwrap()
                == sop_instance_uid
        })
        .collect::<Vec<_>>();

    if dcm_files.len() == 0 {
        return Err("No instance found".into());
    }

    let dcm = FileDicomObject::open_file(dcm_files[0])?;
    Ok(dcm)
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
            .wrap(middleware::Compress::default())
            .app_data(web::Data::new(dicomweb_server::DicomWebServer {
                search_instances: search_instances,
                search_series: search_series,
                search_study: search_study,
                retrieve_study: retrieve_study,
                retrieve_series: retrieve_series,
                retrieve_instance: retrieve_instance,
                store_instances: store_instances,
            }))
            .configure(dicomweb_config)
    })
    .bind(SELF_URL)?
    .run()
    .await
}
