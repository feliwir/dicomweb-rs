use dicom_object::InMemDicomObject;

use crate::QidoStudyQuery;

pub fn study_filter(_dcm: &InMemDicomObject, _query: &QidoStudyQuery) -> bool {
    true
}
