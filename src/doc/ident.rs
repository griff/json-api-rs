use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::mem;
use async_trait::async_trait;

use crate::doc::{Data, Document, Object, PrimaryData};
use crate::query::Query;
use crate::sealed::Sealed;
use crate::value::{Key, Map, Set, Value};
use crate::view::{Resolver, ResolveError};

use super::de::DocGraph;

/// Identifies an individual resource. Commonly found in an object's relationships.
///
/// Identifiers share their [equality] and [hashing] behavior with [`Object`]. For more
/// information, check out the *[resource identifier objects]* section of the
/// JSON API specification.
///
/// [`Object`]: ./struct.Object.html
/// [equality]: ./struct.Object.html#equality
/// [hashing]: ./struct.Object.html#hashing
/// [resource identifier objects]: https://goo.gl/vgfzru
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Identifier {
    /// A string that contains a unique identfier for this resource type (`kind`). For
    /// more information, check out the *[identification]* section of the JSON API
    /// specification.
    ///
    /// [identification]: https://goo.gl/3s681i
    pub id: String,

    /// Describes resources that share common attributes and relationships. This field is
    /// derived from the `type` field if the identifier is deserialized. For more
    /// information, check out the *[identification]* section of the JSON API
    /// specification.
    ///
    /// [identification]: https://goo.gl/3s681i
    #[serde(rename = "type")]
    pub kind: Key,

    /// Non-standard meta information. If this value of this field is empty, it will not
    /// be serialized. For more information, check out the *[meta information]* section
    /// of the JSON API specification.
    ///
    /// [meta information]: https://goo.gl/LyrGF8
    #[serde(default, skip_serializing_if = "Map::is_empty")]
    pub meta: Map,

    /// Private field for backwards compatibility.
    #[serde(skip)]
    _ext: (),
}

impl Identifier {
    /// Returns a new `Identifier`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::value::fields::ParseKeyError;
    /// #
    /// # fn example() -> Result<(), ParseKeyError> {
    /// use json_api::doc::Identifier;
    /// let mut ident = Identifier::new("users".parse()?, "1".to_owned());
    /// # Ok(())
    /// # }
    /// #
    /// # fn main() {
    /// # example().unwrap();
    /// # }
    /// ```
    pub fn new(kind: Key, id: String) -> Self {
        Identifier {
            id,
            kind,
            meta: Default::default(),
            _ext: (),
        }
    }
}

impl Eq for Identifier {}

impl From<Object> for Identifier {
    fn from(object: Object) -> Self {
        let Object { id, kind, meta, .. } = object;
        let mut ident = Identifier::new(kind, id);

        ident.meta = meta;
        ident
    }
}

impl<'a> From<&'a Object> for Identifier {
    fn from(object: &'a Object) -> Self {
        object.clone().into()
    }
}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.kind.hash(state);
    }
}

impl PartialEq for Identifier {
    fn eq(&self, rhs: &Identifier) -> bool {
        self.id == rhs.id && self.kind == rhs.kind
    }
}

impl PartialEq<Object> for Identifier {
    fn eq(&self, rhs: &Object) -> bool {
        self.id == rhs.id && self.kind == rhs.kind
    }
}

#[async_trait]
impl Resolver<Identifier> for Identifier {
    type Context = ();
    async fn resolve(mut self, _: Option<&Query>, _: &Self::Context) -> Result<Document<Identifier>, ResolveError> {
        let meta = mem::replace(&mut self.meta, Default::default());

        Ok(Document::Ok {
            meta,
            data: Data::Member(Box::new(Some(self))),
            included: Default::default(),
            jsonapi: Default::default(),
            links: Default::default(),
        })
    }
}

#[async_trait]
impl Resolver<Identifier> for Vec<Identifier> {
    type Context = ();
    async fn resolve(self, _: Option<&Query>, _: &Self::Context) -> Result<Document<Identifier>, ResolveError> {
        Ok(Document::Ok {
            data: Data::Collection(self),
            included: Default::default(),
            jsonapi: Default::default(),
            links: Default::default(),
            meta: Default::default(),
        })
    }
}

impl PrimaryData for Identifier {
    fn flatten(self, incl: &Set<Object>) -> Value {
        incl.into_iter()
            .find(|item| self == **item)
            .map(|item| item.clone().flatten(incl))
            .unwrap_or_else(|| self.id.clone().into())
    }
    fn deserializer<'de>(self, included: &'de Set<Object>) -> DocGraph<'de> {
        DocGraph::id(self, included)
    }
}

impl Sealed for Identifier {}
