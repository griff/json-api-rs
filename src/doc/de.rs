
use serde::forward_to_deserialize_any;
use serde::de::{Deserialize, Deserializer, DeserializeSeed, EnumAccess, IntoDeserializer};
use serde::de::{MapAccess, SeqAccess, VariantAccess, Visitor, Unexpected};
use serde_json::{self, Value as JsonValue};

use crate::doc::{Data, NewObject, Object, Relationship};
use crate::value::{Key, Map, Set, Value};
use crate::value::collections::map::IntoIter;

use super::Identifier;

pub struct DocGraph<'a> {
    kind: Key,
    id: Option<String>,
    attributes: Option<Map<Key, Value>>,
    relationships: Option<(usize, Map<Key, Relationship>)>,
    included: &'a Set<Object>,
} 

impl<'a> DocGraph<'a> {
    pub(crate) fn id(id: Identifier, included: &'a Set<Object>) -> DocGraph<'a> {
        DocGraph {
            kind: id.kind,
            id: Some(id.id),
            attributes: None,
            relationships: None,
            included,
        }
    }

    pub(crate) fn object(object: Object, included: &'a Set<Object>) -> DocGraph<'a> {
        let len = object.relationships.iter().filter(|(_, v)| v.data.is_some()).count();
        DocGraph {
            kind: object.kind,
            id: Some(object.id),
            attributes: Some(object.attributes),
            relationships: Some((len, object.relationships)),
            included,
        }
    }

    pub(crate) fn new_object(object: NewObject, included: &'a Set<Object>) -> DocGraph<'a> {
        let len = object.relationships.iter().filter(|(_, v)| v.data.is_some()).count();
        DocGraph {
            kind: object.kind,
            id: object.id, 
            attributes: Some(object.attributes), 
            relationships: Some((len, object.relationships)),
            included,
        }
    }

    fn loopup(id: Identifier, included: &'a Set<Object>) -> DocGraph<'a> {
        included.into_iter()
            .find(|item| id == **item)
            .map(|item| DocGraph::object(item.clone(), included))
            .unwrap_or_else(|| DocGraph::id(id.clone(), included))
    }

    fn len(&self) -> usize {
        self.attributes.as_ref().map_or(0, |a| a.len())
            + self.relationships.as_ref().map_or(0, |(len, _)| *len)
            + 2    
    }
}


impl<'de> Deserializer<'de> for DocGraph<'de> {
    type Error = serde_json::Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visit_object(self, visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.id {
            Some(v) => visitor.visit_string(v),
            _ => Err(serde::de::Error::invalid_type(Unexpected::Unit, &visitor)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.id {
            Some(v) => visitor.visit_string(v),
            _ => Err(serde::de::Error::invalid_type(Unexpected::Unit, &visitor)),
            //_ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 unit unit_struct
        seq tuple tuple_struct struct map
    }
    /*
    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier
    }
    */

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

impl<'de> EnumAccess<'de> for DocGraph<'de> {
    type Error = serde_json::Error;
    type Variant = DocGraph<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, DocGraph<'de>), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.kind.into_deserializer();
        seed.deserialize(variant).map(|v| (v, self))
    }
}

impl<'de> VariantAccess<'de> for DocGraph<'de> {
    type Error = serde_json::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Deserialize::deserialize(self)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(serde::de::Error::invalid_type(
            Unexpected::Map,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::Deserializer::deserialize_any(MapDocGraph::from(self), visitor)
    }
}

fn visit_object<'de, V>(object: DocGraph<'de>, visitor: V) -> Result<V::Value, serde_json::Error>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapDocGraph::from(object);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}


pub struct MapDocGraph<'de> {
    data: DocData,
    included: &'de Set<Object>,
} 

impl<'de> MapDocGraph<'de> {
    fn len(&self) -> usize {
        match &self.data {
            DocData::Kind(_key, _id, attr, relations) => {
                attr.as_ref().map_or(0, |a| a.len())
                    + relations.as_ref().map_or(0, |(len, _)| *len)
                    + 2
            }
            DocData::Id(_id, attr, relations) => {
                attr.as_ref().map_or(0, |a| a.len())
                    + relations.as_ref().map_or(0, |(len, _)| *len)
                    + 1
            }
            DocData::Attributes(_value, attr, relations) => {
                attr.len() + relations.as_ref().map_or(0, |(len, _)| *len)
            }
            DocData::Relationships(_value, len, _relations) => {
                *len
            }
            DocData::Done => {
                0
            }
        }        
    }
}

impl<'de> From<DocGraph<'de>> for MapDocGraph<'de> {
    fn from(s: DocGraph<'de>) -> MapDocGraph<'de> {
        MapDocGraph {
            data: DocData::Kind(s.kind, s.id, s.attributes, s.relationships),
            included: s.included
        }
    }
}

enum DocData {
    Kind(Key, Option<String>, Option<Map>, Option<(usize, Map<Key, Relationship>)>),
    Id(String, Option<Map>, Option<(usize, Map<Key, Relationship>)>),
    Attributes(Option<Value>, IntoIter<Key, Value>, Option<(usize, Map<Key, Relationship>)>),
    Relationships(Option<Relationship>, usize, IntoIter<Key, Relationship>),
    Done,
}

impl DocData {
    fn take(&mut self) -> DocData {
        std::mem::replace(self, DocData::Done)
    }
}

impl<'de> MapAccess<'de> for MapDocGraph<'de> {
    type Error = serde_json::Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        loop {
            match self.data.take() {
                data @ DocData::Kind(_, _, _, _) => {
                    self.data = data;
                    return seed.deserialize(Key::from_raw("type".to_string())).map(Some);
                },
                data @ DocData::Id(_, _, _) => {
                    self.data = data;
                    return seed.deserialize(Key::from_raw("id".to_string())).map(Some);
                },
                DocData::Attributes(_value, mut attr, relations) => {
                    match (attr.next(), relations) {
                        (Some((key, value)), relations) => {
                            self.data = DocData::Attributes(Some(value), attr, relations);
                            return seed.deserialize(key).map(Some);
                        },
                        (None, Some((len, relations))) => {
                            self.data = DocData::Relationships(None, len, relations.into_iter());
                        },
                        (None, None) => {
                            self.data = DocData::Done;
                        }
                    }
                },
                DocData::Relationships(_value, len, mut relations) => {
                    match relations.next() {
                        Some((key, value)) => {
                            if value.data.is_some() {
                                self.data = DocData::Relationships(Some(value), len - 1, relations);
                                return seed.deserialize(key).map(Some);
                            } else {
                                self.data = DocData::Relationships(None, len, relations);
                            }
                        },
                        None => {
                            self.data = DocData::Done;
                            return Ok(None);
                        },
                    }
                },
                DocData::Done => {
                    return Ok(None);
                }
            }
        }
    }   

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.data.take() {
            DocData::Kind(key, id, attr, relations) => {
                match (id, attr, relations) {
                    (Some(id), attr, relations) => {
                        self.data = DocData::Id(id, attr, relations);
                    },
                    (None, Some(attr), relations) => {
                        self.data = DocData::Attributes(None, attr.into_iter(), relations);
                    },
                    (None, None, Some((len, relations))) => {
                        self.data = DocData::Relationships(None, len, relations.into_iter());
                    }
                    (None, None, None) => {
                        self.data = DocData::Done;
                    }
                }
                return seed.deserialize(key);
            },
            DocData::Id(id, attr, relations) => {
                match (attr, relations) {
                    (Some(attr), relations) => {
                        self.data = DocData::Attributes(None, attr.into_iter(), relations);
                    },
                    (None, Some((len, relations))) => {
                        self.data = DocData::Relationships(None, len, relations.into_iter());
                    }
                    (None, None) => {
                        self.data = DocData::Done;
                    }
                }
                return Ok(seed.deserialize(JsonValue::String(id))?);
            },
            DocData::Attributes(value, attr, relations) => {
                self.data = DocData::Attributes(None, attr, relations);
                match value {
                    Some(value) => seed.deserialize(value),
                    None => Err(serde::de::Error::custom("attribute value is missing")),
                }
            },
            DocData::Relationships(value, len, relations) => {
                self.data = DocData::Relationships(None, len, relations);
                match value {
                    Some(value) => {
                        match value.data.unwrap() {
                            Data::Member(data) => match *data {
                                Some(id) => {
                                    let value = DocGraph::loopup(id, self.included);
                                    Ok(seed.deserialize(value)?)
                                },
                                None => Ok(seed.deserialize(Value::Null)?),
                            },
                            Data::Collection(data) => {
                                let iter = data.into_iter().map(|id| DocGraph::loopup(id, self.included));
                                Ok(seed.deserialize(SeqDeserializer::iter(iter))?)
                            }
                        }
                    }
                    None => Err(serde::de::Error::custom("relationship value is missing")),
                }
            },
            DocData::Done => {
                Err(serde::de::Error::custom("premature done is missing"))
            },
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match &self.data {
            DocData::Kind(_key, _id, attr, relations) => {
                Some(attr.as_ref().map_or(0, |a| a.len())
                    + relations.as_ref().map_or(0, |(len, _)| *len)
                    + 2)
            }
            DocData::Id(_id, attr, relations) => {
                Some(attr.as_ref().map_or(0, |a| a.len())
                    + relations.as_ref().map_or(0, |(len, _)| *len)
                    + 1)
            }
            DocData::Attributes(_value, attr, relations) => {
                match attr.size_hint() {
                    (lower, Some(upper)) if lower == upper => Some(upper + relations.as_ref().map_or(0, |(len, _)| *len)),
                    _ => None,
                }
            }
            DocData::Relationships(_value, len, _relations) => {
                Some(*len)
            }
            DocData::Done => {
                None
            }
        }
    }
}

impl<'de> Deserializer<'de> for MapDocGraph<'de> {
    type Error = serde_json::Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

pub(crate) struct SeqDeserializer<'de, I>
    where I:Iterator<Item=DocGraph<'de>>
{
    iter: I,
}
impl<'de, I> SeqDeserializer<'de, I>
    where I: Iterator<Item=DocGraph<'de>> + ExactSizeIterator
{
    pub(crate) fn iter(iter: I) -> SeqDeserializer<'de, I> {
        SeqDeserializer{ iter }
    }
}

/*
impl<'de> SeqDeserializer<std::vec::IntoIter<DocGraph<'de>>> {
    fn new(vec: Vec<DocGraph<'de>>) -> SeqDeserializer<std::vec::IntoIter<DocGraph<'de>>> {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}
*/

impl<'de, I> serde::Deserializer<'de> for SeqDeserializer<'de, I>
    where I: Iterator<Item=DocGraph<'de>> + ExactSizeIterator
{
    type Error = serde_json::Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let len = self.iter.len();
        if len == 0 {
            visitor.visit_unit()
        } else {
            let ret = visitor.visit_seq(&mut self)?;
            let remaining = self.iter.len();
            if remaining == 0 {
                Ok(ret)
            } else {
                Err(serde::de::Error::invalid_length(
                    len,
                    &"fewer elements in array",
                ))
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}


impl<'de, I> SeqAccess<'de> for SeqDeserializer<'de, I>
    where I: Iterator<Item=DocGraph<'de>>,
{
    type Error = serde_json::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}




macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self.parse() {
                Ok(integer) => visitor.$visit(integer),
                Err(_) => visitor.visit_string(self.into()),
            }
        }
    };
}

impl<'de> Deserializer<'de> for Key {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_string(self.into())
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Map keys cannot be null.
        visitor.visit_some(self)
    }

    deserialize_integer_key!(deserialize_i8 => visit_i8);
    deserialize_integer_key!(deserialize_i16 => visit_i16);
    deserialize_integer_key!(deserialize_i32 => visit_i32);
    deserialize_integer_key!(deserialize_i64 => visit_i64);
    deserialize_integer_key!(deserialize_u8 => visit_u8);
    deserialize_integer_key!(deserialize_u16 => visit_u16);
    deserialize_integer_key!(deserialize_u32 => visit_u32);
    deserialize_integer_key!(deserialize_u64 => visit_u64);

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        (&self).into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        bool f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}
