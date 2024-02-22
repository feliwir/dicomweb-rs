use dicom::dictionary_std::tags;
use dicom_object::InMemDicomObject;

use crate::QidoStudyQuery;

pub fn study_filter(dcm: &InMemDicomObject, query: &QidoStudyQuery) -> bool {
    true
}
