const ELEMENT_COUNT: usize = 128;
const RAW_ELEMENT_SIZE: usize = std::mem::size_of::<f64>();
const RAW_SIZE: usize = ELEMENT_COUNT * RAW_ELEMENT_SIZE;

#[derive(Clone, Debug, diesel::AsExpression, diesel::FromSqlRow, PartialEq)]
#[sql_type = "diesel::sql_types::Binary"]
pub struct FaceEncoding(dlib_face_recognition::FaceEncoding);

impl FaceEncoding {
    pub fn distance(&self, other: &FaceEncoding) -> f64 {
        self.0.distance(&other.0)
    }
}

impl From<dlib_face_recognition::FaceEncoding> for FaceEncoding {
    fn from(encoding: dlib_face_recognition::FaceEncoding) -> Self {
        Self(encoding)
    }
}

impl std::fmt::Display for FaceEncoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl serde::Serialize for FaceEncoding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let elements = self.0.to_elements();
        let mut seq = serializer.serialize_seq(Some(elements.len()))?;
        for element in elements.iter() {
            seq.serialize_element(element)?;
        }
        seq.end()
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unexpected SQL FaceEncoding length {0}, should be {1}")]
struct BadFaceEncodingLengthError(usize, usize);

impl<ST, DB> diesel::deserialize::FromSql<ST, DB> for FaceEncoding
where
    DB: diesel::backend::Backend,
    *const [u8]: diesel::deserialize::FromSql<ST, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        let slice_ptr = <*const [u8] as diesel::deserialize::FromSql<ST, DB>>::from_sql(bytes)?;
        // We know that the pointer impl will never return null
        let bytes = unsafe { &*slice_ptr };

        if bytes.len() == RAW_SIZE {
            let mut elements = [0f64; ELEMENT_COUNT];

            for (e, chunk) in bytes.chunks(RAW_ELEMENT_SIZE).enumerate() {
                let mut element_bytes = [0u8; RAW_ELEMENT_SIZE];
                element_bytes.copy_from_slice(chunk);
                elements[e] = f64::from_le_bytes(element_bytes);
            }

            Ok(FaceEncoding(dlib_face_recognition::FaceEncoding::new(
                &elements,
            )))
        } else {
            Err(Box::new(BadFaceEncodingLengthError(bytes.len(), RAW_SIZE))
                as Box<dyn std::error::Error + Send + Sync>)
        }
    }
}

impl<DB> diesel::serialize::ToSql<diesel::sql_types::Binary, DB> for FaceEncoding
where
    DB: diesel::backend::Backend,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, DB>,
    ) -> diesel::serialize::Result {
        let elements = self.0.to_elements();

        let mut bytes = [0u8; RAW_SIZE];
        for (e, chunk) in bytes.chunks_mut(RAW_ELEMENT_SIZE).enumerate() {
            chunk.copy_from_slice(&elements[e].to_le_bytes());
        }

        out.write_all(&bytes)
            .map(|_| diesel::serialize::IsNull::No)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}
