use std::io::{Read, Write};

use serde::de::DeserializeOwned;
use serde_json;
use thiserror::Error;
use serde_path_to_error::deserialize;

use crate::doc::{Data, Document, DocumentError, PrimaryData, Object};
use crate::doc::de::SeqDeserializer;
//use crate::error::Error;
use crate::query::Query;
use crate::value::Value;
use crate::view::{Resolver, ResolveError};

#[derive(Error, Debug)]
pub enum ParseDocumentError {
    #[error("json error parsing document {0}")]
    Json(#[source] #[from] serde_json::Error),
    #[error("json error parsing document {0}")]
    JsonEE(#[source] #[from] serde_path_to_error::Error<serde_json::Error>),
    #[error("document errors {0}")]
    Document(#[source] #[from] DocumentError),
}

/// Interpret a `Document<T>` as a type `U`.
pub fn from_doc<T, U>(doc: Document<T>) -> Result<U, ParseDocumentError>
where
    T: PrimaryData,
    U: DeserializeOwned,
{
    match doc {
        Document::Ok { data, included, .. } => {
            match data {
                Data::Member(data) => match *data {
                    Some(item) => Ok(deserialize(item.deserializer(&included))?),
                    None => Ok(deserialize(Value::Null)?),
                },
                Data::Collection(data) => {
                    Ok(deserialize(SeqDeserializer::iter(
                        data.into_iter().map(|item| item.deserializer(&included))))?)
                },
            }
        }
        Document::Err(e) => return Err(e)?,
    }
}

/// Deserialize a `Document<T>` from an IO stream of JSON text and then
/// iterpret it as a type `U`.
pub fn from_reader<R, T, U>(data: R) -> Result<U, ParseDocumentError>
where
    R: Read,
    T: PrimaryData,
    U: DeserializeOwned,
{
    from_doc::<T, _>(serde_json::from_reader(data)?)
}

/// Deserialize a `Document<T>` from bytes of JSON text and then iterpret it as
/// a type `U`.
pub fn from_slice<T, U>(data: &[u8]) -> Result<U, ParseDocumentError>
where
    T: PrimaryData,
    U: DeserializeOwned,
{
    from_doc::<T, _>(serde_json::from_slice(data)?)
}

/// Deserialize a `Document<T>` from a string of JSON text and then iterpret it
/// as a type `U`.
pub fn from_str<T, U>(data: &str) -> Result<U, ParseDocumentError>
where
    T: PrimaryData,
    U: DeserializeOwned,
{
    from_doc::<T, _>(serde_json::from_str(data)?)
}

/// Render type `T` as a `Document<U>`.
pub async fn to_doc<T, U>(value: T, query: Option<&Query>, ctx: &T::Context) -> Result<Document<U>, ResolveError>
where
    T: Resolver<U>,
    U: PrimaryData,
{
    value.resolve(query, ctx).await
}

pub async fn to_doc_object<T>(value: T, query: Option<&Query>, ctx: &T::Context) -> Result<Document<Object>, ResolveError>
where
    T: Resolver<Object>,
{
    value.resolve(query, ctx).await
}

#[derive(Error, Debug)]
pub enum SerializeDocumentError {
    #[error("json error serializing document {0}")]
    Json(#[source] #[from] serde_json::Error),
    #[error("resolve error in document {0}")]
    Resolve(#[source] #[from] ResolveError),
}

/// Render type `T` as a `Document<U>` and then serialize it as a string of
/// JSON.
pub async fn to_string<T, U>(value: T, query: Option<&Query>, ctx: &T::Context) -> Result<String, SerializeDocumentError>
where
    T: Resolver<U>,
    U: PrimaryData,
{
    Ok(serde_json::to_string(&to_doc(value, query, ctx).await?)?)
}

/// Render type `T` as a `Document<U>` and then serialize it as a
/// pretty-printed string of JSON.
pub async fn to_string_pretty<T, U>(value: T, query: Option<&Query>, ctx: &T::Context) -> Result<String, SerializeDocumentError>
where
    T: Resolver<U>,
    U: PrimaryData,
{
    Ok(serde_json::to_string_pretty(&to_doc(value, query, ctx).await?)?)
}

/// Render type `T` as a `Document<U>` and then serialize it as a JSON byte
/// vector.
pub async fn to_vec<T, U>(value: T, query: Option<&Query>, ctx: &T::Context) -> Result<Vec<u8>, SerializeDocumentError>
where
    T: Resolver<U>,
    U: PrimaryData,
{
    Ok(serde_json::to_vec(&to_doc(value, query, ctx).await?)?)
}

/// Render type `T` as a `Document<U>` and then serialize it as a
/// pretty-printed JSON byte vector.
pub async fn to_vec_pretty<T, U>(value: T, query: Option<&Query>, ctx: &T::Context) -> Result<Vec<u8>, SerializeDocumentError>
where
    T: Resolver<U>,
    U: PrimaryData,
{
    Ok(serde_json::to_vec_pretty(&to_doc(value, query, ctx).await?)?)
}

/// Render type `T` as a `Document<U>` and then serialize it as JSON into the
/// IO stream.
pub async fn to_writer<W, T, U>(writer: W, value: T, query: Option<&Query>, ctx: &T::Context) -> Result<(), SerializeDocumentError>
where
    W: Write,
    T: Resolver<U>,
    U: PrimaryData,
{
    serde_json::to_writer(writer, &to_doc(value, query, ctx).await?)?;
    Ok(())
}

/// Render type `T` as a `Document<U>` and then serialize it as pretty-printed
/// JSON into the IO stream.
pub async fn to_writer_pretty<W, T, U>(writer: W, value: T, query: Option<&Query>, ctx: &T::Context) -> Result<(), SerializeDocumentError>
where
    W: Write,
    T: Resolver<U>,
    U: PrimaryData,
{
    serde_json::to_writer_pretty(writer, &to_doc(value, query, ctx).await?)?;
    Ok(())
}
