use async_trait::async_trait;
use thiserror::Error;

use crate::doc::{Data, Document, PrimaryData};
//use crate::error::Error;
use crate::query::Query;
use crate::value::{ParseKeyError, ValueError};



#[derive(Error, Debug)]
pub enum ResolveError {
    #[error("parse key error")]
    ParseKey(#[source] #[from] ParseKeyError),
    #[error("value error")]
    Value(#[source] #[from] ValueError),
    #[error("invalid link URI")]
    Link(#[source] #[from] http::uri::InvalidUri),
    #[error("error {0}")]
    Custom(String)
}


#[async_trait]
pub trait Resolver<T: PrimaryData> {
    type Context;

    /// Attempts to render the given type as a document.
    ///
    /// Types that implement the [`Resource`] trait via the [`resource!`] macro can use
    /// the optional query argument to match object field-sets and included resources
    /// with what is present in the query.
    ///
    /// If a query does not have a matching field-set for a given type and the type in
    /// question is a part of the document's primary data or included resources, each
    /// attribute specified in the type's [`resource!`] macro invocation will be used.
    ///
    /// [`Resource`]: ../trait.Resource.html
    /// [`resource!`]: ../macro.resource.html
    async fn resolve(self, query: Option<&Query>, ctx: &Self::Context) -> Result<Document<T>, ResolveError>;
}

#[async_trait]
impl<D, T> Resolver<D> for Option<T>
where
    D: PrimaryData + Send + Sync + 'static,
    T: Resolver<D> + Sized + Send + Sync,
    <T as Resolver<D>>::Context: Send + Sync,
{
    type Context = T::Context;
    async fn resolve(self, query: Option<&Query>, ctx: &Self::Context) -> Result<Document<D>, ResolveError> {
        match self {
            Some(value) => value.resolve(query, ctx).await,
            None => Ok(Document::Ok {
                data: Data::Member(Box::new(None)),
                included: Default::default(),
                jsonapi: Default::default(),
                links: Default::default(),
                meta: Default::default(),
            }),
        }
    }
}
