use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

/// Deserialize string into T
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    String::deserialize(deserializer)?
        .parse::<T>()
        .map_err(|e| D::Error::custom(e.to_string()))
}

/// Serialize from T into string
pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: std::string::ToString,
{
    value.to_string().serialize(serializer)
}
