//! Functions that convert types to and from a `Value`.

use serde::de::{self, DeserializeOwned};
use serde::ser::Serialize;
use serde_json::{self, Value as JsonValue};
use thiserror::Error;

use crate::value::{Value, ParseKeyError};

#[derive(Error, Debug)]
pub enum ValueError {
    #[error("json error {0}")]
    Json(#[source] #[from] serde_json::Error),
    #[error("parse key error {0}")]
    ParseKey(#[source] #[from] ParseKeyError),
}

impl serde::de::Error for ValueError {
    fn custom<T>(msg: T) -> Self
        where T: std::fmt::Display
    {
        ValueError::Json(serde_json::Error::custom(msg))
    }

    fn invalid_type(unexp: de::Unexpected, exp: &dyn de::Expected) -> Self {
        ValueError::Json(serde_json::Error::invalid_type(unexp, exp))
    }
}

/// Convert a `T` into a `Value`.
pub fn to_value<T>(value: T) -> Result<Value, ValueError>
where
    T: Serialize,
{
    Ok(from_json(serde_json::to_value(value)?)?)
}

/// Interpret a `Value` as a type `T`.
pub fn from_value<T>(value: Value) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    Ok(T::deserialize(to_json(value))?)
}

pub(crate) fn to_json(value: Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Array(inner) => inner.into_iter().map(to_json).collect(),
        Value::Bool(inner) => JsonValue::Bool(inner),
        Value::Number(inner) => JsonValue::Number(inner),
        Value::Object(inner) => {
            let map = inner
                .into_iter()
                .map(|(k, v)| (String::from(k), to_json(v)))
                .collect();

            JsonValue::Object(map)
        }
        Value::String(inner) => JsonValue::String(inner),
    }
}

pub(crate) fn from_json(value: JsonValue) -> Result<Value, ParseKeyError> {
    match value {
        JsonValue::Null => Ok(Value::Null),
        JsonValue::Array(data) => data.into_iter().map(from_json).collect(),
        JsonValue::Bool(data) => Ok(Value::Bool(data)),
        JsonValue::Number(data) => Ok(Value::Number(data)),
        JsonValue::Object(data) => data.into_iter()
            .map(|(k, v)| Ok((k.parse()?, from_json(v)?)))
            .collect(),
        JsonValue::String(data) => Ok(Value::String(data)),
    }
}
