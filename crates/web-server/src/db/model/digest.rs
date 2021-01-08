const DIGEST_LEN: usize = 16;

#[derive(Clone, Debug, diesel::AsExpression, diesel::FromSqlRow, Eq, std::hash::Hash, PartialEq)]
#[sql_type = "diesel::sql_types::Binary"]
pub struct Digest([u8; DIGEST_LEN]);

impl Digest {
    pub fn compute<T: AsRef<[u8]>>(data: T) -> Self {
        Self(*md5::compute(data))
    }
}

impl std::fmt::Display for Digest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD)
        )
    }
}

impl serde::Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&base64::encode_config(&self.0, base64::URL_SAFE_NO_PAD))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Unexpected SQL digest length {0}, should be {1}")]
struct BadDigestLengthError(usize, usize);

impl<ST, DB> diesel::deserialize::FromSql<ST, DB> for Digest
where
    DB: diesel::backend::Backend,
    *const [u8]: diesel::deserialize::FromSql<ST, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        let slice_ptr = <*const [u8] as diesel::deserialize::FromSql<ST, DB>>::from_sql(bytes)?;
        // We know that the pointer impl will never return null
        let bytes = unsafe { &*slice_ptr };

        if bytes.len() == DIGEST_LEN {
            let mut a = [0u8; DIGEST_LEN];
            a.copy_from_slice(&bytes);
            Ok(Digest(a))
        } else {
            Err(Box::new(BadDigestLengthError(bytes.len(), DIGEST_LEN))
                as Box<dyn std::error::Error + Send + Sync>)
        }
    }
}

impl<DB> diesel::serialize::ToSql<diesel::sql_types::Binary, DB> for Digest
where
    DB: diesel::backend::Backend,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, DB>,
    ) -> diesel::serialize::Result {
        out.write_all(&self.0)
            .map(|_| diesel::serialize::IsNull::No)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}
