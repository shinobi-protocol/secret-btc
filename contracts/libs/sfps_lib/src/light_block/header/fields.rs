use prost::Message;
use serde::{Deserialize, Serialize};

#[derive(prost::Message, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Version {
    /// Block version
    #[prost(uint64, tag = "1")]
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::str")]
    pub block: u64,

    /// App version
    #[prost(uint64, tag = "2")]
    #[schemars(with = "String")]
    #[serde(with = "crate::serde::str", default)]
    pub app: u64,
}

/// BlockID
#[derive(Clone, PartialEq, Message, Deserialize, Serialize, schemars::JsonSchema)]
pub struct BlockId {
    #[prost(bytes, tag = "1")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub hash: Vec<u8>,
    #[prost(message, optional, tag = "2")]
    pub parts: Option<PartSetHeader>,
}

/// Block parts header
#[derive(Clone, Message, PartialEq, Deserialize, Serialize, schemars::JsonSchema)]
pub struct PartSetHeader {
    #[prost(uint32, tag = "1")]
    pub total: u32,
    #[prost(bytes, tag = "2")]
    #[schemars(with = "String")]
    #[serde(serialize_with = "hex::serde::serialize_upper")]
    #[serde(deserialize_with = "hex::serde::deserialize")]
    pub hash: Vec<u8>,
}

// Google protobuf Timestamp and Duration types reimplemented because their comments are turned
// into invalid documentation texts and doctest chokes on them. See https://github.com/danburkert/prost/issues/374
// Prost does not seem to have a way yet to remove documentations defined in protobuf files.
// These structs are defined in gogoproto v1.3.1 at https://github.com/gogo/protobuf/tree/v1.3.1/protobuf/google/protobuf

/// A Timestamp represents a point in time independent of any time zone or local
/// calendar, encoded as a count of seconds and fractions of seconds at
/// nanosecond resolution. The count is relative to an epoch at UTC midnight on
/// January 1, 1970, in the proleptic Gregorian calendar which extends the
/// Gregorian calendar backwards to year one.
///
/// All minutes are 60 seconds long. Leap seconds are "smeared" so that no leap
/// second table is needed for interpretation, using a [24-hour linear
/// smear](https://developers.google.com/time/smear).
///
/// The range is from 0001-01-01T00:00:00Z to 9999-12-31T23:59:59.999999999Z. By
/// restricting to that range, we ensure that we can convert to and from [RFC
/// 3339](https://www.ietf.org/rfc/rfc3339.txt) date strings.
#[derive(Clone, PartialEq, Message, Deserialize, Serialize, schemars::JsonSchema)]
#[serde(from = "serde_timestamp::Rfc3339", into = "serde_timestamp::Rfc3339")]
#[schemars(with = "String")]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}

mod serde_timestamp {
    //! Serialize/deserialize Timestamp type from and into string:
    use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};

    use super::Timestamp;
    use chrono::{DateTime, LocalResult, TimeZone, Utc};
    use serde::ser::Error;

    /// Helper struct to serialize and deserialize Timestamp into an RFC3339-compatible string
    /// This is required because the serde `with` attribute is only available to fields of a struct but
    /// not the whole struct.
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(transparent)]
    pub struct Rfc3339(#[serde(with = "super::serde_timestamp")] Timestamp);

    impl From<Timestamp> for Rfc3339 {
        fn from(value: Timestamp) -> Self {
            Rfc3339(value)
        }
    }
    impl From<Rfc3339> for Timestamp {
        fn from(value: Rfc3339) -> Self {
            value.0
        }
    }

    /// Deserialize string into Timestamp
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value_string = String::deserialize(deserializer)?;
        let value_datetime = DateTime::parse_from_rfc3339(value_string.as_str())
            .map_err(|e| D::Error::custom(format!("{}", e)))?;
        Ok(Timestamp {
            seconds: value_datetime.timestamp(),
            nanos: value_datetime.timestamp_subsec_nanos() as i32,
        })
    }

    /// Serialize from Timestamp into string
    pub fn serialize<S>(value: &Timestamp, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.nanos < 0 {
            return Err(S::Error::custom("invalid nanoseconds in time"));
        }
        match Utc.timestamp_opt(value.seconds, value.nanos as u32) {
            LocalResult::None => Err(S::Error::custom("invalid time")),
            LocalResult::Single(t) => Ok(to_rfc3339_nanos(&t)),
            LocalResult::Ambiguous(_, _) => Err(S::Error::custom("ambiguous time")),
        }?
        .serialize(serializer)
        //}
    }

    /// Serialization helper for converting a `DateTime<Utc>` object to a string.
    ///
    /// This reproduces the behavior of Go's `time.RFC3339Nano` format,
    /// ie. a RFC3339 date-time with left-padded subsecond digits without
    ///     trailing zeros and no trailing dot.
    pub fn to_rfc3339_nanos(t: &DateTime<Utc>) -> String {
        use chrono::format::{Fixed, Item, Numeric::*, Pad::Zero};

        const PREFIX: &[Item<'_>] = &[
            Item::Numeric(Year, Zero),
            Item::Literal("-"),
            Item::Numeric(Month, Zero),
            Item::Literal("-"),
            Item::Numeric(Day, Zero),
            Item::Literal("T"),
            Item::Numeric(Hour, Zero),
            Item::Literal(":"),
            Item::Numeric(Minute, Zero),
            Item::Literal(":"),
            Item::Numeric(Second, Zero),
        ];

        const NANOS: &[Item<'_>] = &[Item::Fixed(Fixed::Nanosecond)];

        // Format as RFC339 without nanoseconds nor timezone marker
        let prefix = t.format_with_items(PREFIX.iter());

        // Format nanoseconds with dot, leading zeros, and variable number of trailing zeros
        let nanos = t.format_with_items(NANOS.iter()).to_string();

        // Trim trailing zeros and remove leftover dot if any
        let nanos_trimmed = nanos.trim_end_matches('0').trim_end_matches('.');

        format!("{}{}Z", prefix, nanos_trimmed)
    }

    #[allow(warnings)]
    #[cfg(test)]
    mod test {
        use super::*;
        use serde::{Deserialize, Serialize};

        // The Go code with which the following timestamps
        // were tested is as follows:
        //
        // ```go
        // package main
        //
        // import (
        //     "fmt"
        //     "time"
        // )
        //
        // func main() {
        //     timestamps := []string{
        //         "1970-01-01T00:00:00Z",
        //         "0001-01-01T00:00:00Z",
        //         "2020-09-14T16:33:00Z",
        //         "2020-09-14T16:33:00.1Z",
        //         "2020-09-14T16:33:00.211914212Z",
        //         "2020-09-14T16:33:54.21191421Z",
        //         "2021-01-07T20:25:56.045576Z",
        //         "2021-01-07T20:25:57.039219Z",
        //         "2021-01-07T20:26:05.00509Z",
        //         "2021-01-07T20:26:05.005096Z",
        //         "2021-01-07T20:26:05.0005096Z",
        //     }
        //     for _, timestamp := range timestamps {
        //         ts, err := time.Parse(time.RFC3339Nano, timestamp)
        //         if err != nil {
        //             panic(err)
        //         }
        //         tss := ts.Format(time.RFC3339Nano)
        //         if timestamp != tss {
        //             panic(fmt.Sprintf("\nExpected : %s\nActual   : %s", timestamp, tss))
        //         }
        //     }
        //     fmt.Println("All good!")
        // }
        // ```
        #[test]
        fn json_timestamp_precision() {
            let test_timestamps = vec![
                "1970-01-01T00:00:00Z",
                "0001-01-01T00:00:00Z",
                "2020-09-14T16:33:00Z",
                "2020-09-14T16:33:00.1Z",
                "2020-09-14T16:33:00.211914212Z",
                "2020-09-14T16:33:54.21191421Z",
                "2021-01-07T20:25:56.045576Z",
                "2021-01-07T20:25:57.039219Z",
                "2021-01-07T20:26:05.00509Z",
                "2021-01-07T20:26:05.005096Z",
                "2021-01-07T20:26:05.0005096Z",
            ];

            for timestamp in test_timestamps {
                let json = format!("\"{}\"", timestamp);
                let rfc = serde_json::from_str::<Rfc3339>(&json).unwrap();
                assert_eq!(json, serde_json::to_string(&rfc).unwrap());
            }
        }
    }
}
