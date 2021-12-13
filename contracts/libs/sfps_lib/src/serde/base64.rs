use serde::{de::Error as _, Deserialize, Deserializer, Serializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let string = String::deserialize(deserializer)?;
    base64::decode(&string).map_err(D::Error::custom)
}

pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    let base64_string = base64::encode(value.as_ref());
    serializer.serialize_str(&base64_string)
}
