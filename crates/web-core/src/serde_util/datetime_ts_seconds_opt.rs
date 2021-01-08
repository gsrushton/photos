use chrono::NaiveDateTime;

pub fn serialize<S>(v: &Option<NaiveDateTime>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match v {
        Some(v) => s.serialize_some(&v.timestamp()),
        None => s.serialize_none(),
    }
}

pub fn deserialize<'de, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Option<NaiveDateTime>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a unix timestamp")
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            NaiveDateTime::from_timestamp_opt(value, 0)
                .ok_or_else(|| E::invalid_value(serde::de::Unexpected::Signed(value), &self))
                .map(|datetime| Some(datetime))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            NaiveDateTime::from_timestamp_opt(value as i64, 0)
                .ok_or_else(|| E::invalid_value(serde::de::Unexpected::Unsigned(value), &self))
                .map(|datetime| Some(datetime))
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            deserializer.deserialize_i64(self)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(None)
        }
    }

    d.deserialize_option(Visitor)
}
