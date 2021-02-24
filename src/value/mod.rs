//! Represent and interact with JSON API values.

pub(crate) mod convert;

pub mod collections;
pub mod fields;

use std::borrow::Cow;
use std::cmp::PartialEq;
use std::fmt::{self, Formatter};
use std::iter::FromIterator;
use std::str::FromStr;

use serde::forward_to_deserialize_any;
use serde::de::{self, Deserialize, Deserializer, DeserializeSeed, EnumAccess};
use serde::de::{Expected, IntoDeserializer, MapAccess, SeqAccess, VariantAccess};
use serde::de::{Visitor, Unexpected};
use serde::ser::{Serialize, Serializer};

//use crate::error::Error;

pub use serde_json::value::Number;

pub use self::collections::{Map, Set};
pub use self::convert::{from_value, to_value, ValueError};
#[doc(no_inline)]
pub use self::fields::{Key, ParseKeyError, Path};

/// Represents any valid JSON API value.
///
/// Like [`serde_json::Value`], but with spec compliance baked into the type
/// system.
///
/// [`serde_json::Value`]: https://docs.serde.rs/serde_json/enum.Value.html
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// A null value.
    Null,

    /// An array of values.
    Array(Vec<Value>),

    /// A boolean value.
    Bool(bool),

    /// An integer or floating point value.
    Number(Number),

    /// A JSON object as a hash table with consistent order. Keys are
    /// guarenteed to be a valid [member name].
    ///
    /// [member name]: http://jsonapi.org/format/#document-member-names
    Object(Map),

    /// A string value.
    String(String),
}

impl Value {
    /// Optionally get the underlying vector as a slice. Returns `None` if the
    /// `Value` is not an array.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let data = vec![true.into(), false.into()];
    /// let array = Value::Array(data.clone());
    /// let boolean = Value::Bool(true);
    ///
    /// assert_eq!(array.as_array(), Some(data.as_slice()));
    /// assert_eq!(boolean.as_array(), None);
    /// # }
    /// ```
    pub fn as_array(&self) -> Option<&[Value]> {
        match *self {
            Value::Array(ref inner) => Some(inner),
            _ => None,
        }
    }

    /// Optionally get the underlying vector as a mutable slice. Returns `None`
    /// if the `Value` is not an array.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let mut data = vec![true.into(), false.into()];
    /// let mut array = Value::Array(data.clone());
    /// let mut boolean = Value::Bool(true);
    ///
    /// assert_eq!(array.as_array_mut(), Some(data.as_mut_slice()));
    /// assert_eq!(boolean.as_array_mut(), None);
    /// # }
    /// ```
    pub fn as_array_mut(&mut self) -> Option<&mut [Value]> {
        match *self {
            Value::Array(ref mut inner) => Some(inner),
            _ => None,
        }
    }

    /// Optionally get the inner boolean value. Returns `None` if the `Value` is
    /// not a boolean.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let boolean = Value::Bool(true);
    /// let number = Value::from(3.14);
    ///
    /// assert_eq!(boolean.as_bool(), Some(true));
    /// assert_eq!(number.as_bool(), None);
    /// # }
    /// ```
    pub fn as_bool(&self) -> Option<bool> {
        match *self {
            Value::Bool(inner) => Some(inner),
            _ => None,
        }
    }

    /// Returns `Some(())` if the `Value` is null.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let null = Value::Null;
    /// let text = Value::String("Hello, World!".to_owned());
    ///
    /// assert_eq!(null.as_null(), Some(()));
    /// assert_eq!(text.as_null(), None);
    /// # }
    /// ```
    pub fn as_null(&self) -> Option<()> {
        match *self {
            Value::Null => Some(()),
            _ => None,
        }
    }

    /// Optionally get a reference to the inner map. Returns `None` if the
    /// `Value` is not an object.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::value::{Map, Value};
    /// #
    /// # fn main() {
    /// let data = Map::new();
    /// let object = Value::Object(data.clone());
    /// let number = Value::from(3.14);
    ///
    /// assert_eq!(object.as_object(), Some(&data));
    /// assert_eq!(number.as_object(), None);
    /// # }
    /// ```
    pub fn as_object(&self) -> Option<&Map> {
        match *self {
            Value::Object(ref inner) => Some(inner),
            _ => None,
        }
    }

    /// Optionally get a mutable reference to the inner map. Returns `None` if
    /// the `Value` is not an object.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::value::{Map, Value};
    /// #
    /// # fn main() {
    /// let mut data = Map::new();
    /// let mut object = Value::Object(data.clone());
    /// let mut number = Value::from(3.14);
    ///
    /// assert_eq!(object.as_object_mut(), Some(&mut data));
    /// assert_eq!(number.as_object_mut(), None);
    /// # }
    /// ```
    pub fn as_object_mut(&mut self) -> Option<&mut Map> {
        match *self {
            Value::Object(ref mut inner) => Some(inner),
            _ => None,
        }
    }

    /// Optionally get the underlying string as a string slice. Returns `None`
    /// if the `Value` is not a string.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let data = "Hello, World!";
    /// let string = Value::String(data.to_owned());
    /// let number = Value::from(3.14);
    ///
    /// assert_eq!(string.as_str(), Some(data));
    /// assert_eq!(number.as_str(), None);
    /// # }
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref inner) => Some(inner),
            _ => None,
        }
    }

    /// Optionally get the underlying number as an `f64`. Returns `None` if the
    /// `Value` cannot be represented as an `f64`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let number = Value::from(3.14);
    /// let string = Value::String("Hello, World!".to_owned());
    ///
    /// assert_eq!(number.as_f64(), Some(3.14));
    /// assert_eq!(string.as_f64(), None);
    /// # }
    /// ```
    pub fn as_f64(&self) -> Option<f64> {
        match *self {
            Value::Number(ref n) => n.as_f64(),
            _ => None,
        }
    }

    /// Optionally get the underlying number as an `i64`. Returns `None` if the
    /// `Value` cannot be represented as an `i64`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let integer = Value::from(10);
    /// let float = Value::from(3.14);
    ///
    /// assert_eq!(integer.as_i64(), Some(10));
    /// assert_eq!(float.as_i64(), None);
    /// # }
    /// ```
    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Number(ref n) => n.as_i64(),
            _ => None,
        }
    }

    /// Optionally get the underlying number as an `u64`. Returns `None` if the
    /// `Value` cannot be represented as an `u64`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let positive = Value::from(10);
    /// let negative = Value::from(-10);
    ///
    /// assert_eq!(positive.as_u64(), Some(10));
    /// assert_eq!(negative.as_u64(), None);
    /// # }
    /// ```
    pub fn as_u64(&self) -> Option<u64> {
        match *self {
            Value::Number(ref n) => n.as_u64(),
            _ => None,
        }
    }

    /// Returns true if the `Value` is an array.
    ///
    /// For any `Value` on which `is_array` returns true, [`as_array`] and
    /// [`as_array_mut`] are guaranteed to return a reference to the vector
    /// representing the array.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let mut value = Value::from(vec![1, 2, 3]);
    ///
    /// assert!(value.is_array());
    ///
    /// value.as_array().unwrap();
    /// value.as_array_mut().unwrap();
    /// # }
    /// ```
    ///
    /// [`as_array`]: #method.as_array
    /// [`as_array_mut`]: #method.as_array_mut
    pub fn is_array(&self) -> bool {
        match *self {
            Value::Array(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a boolean.
    ///
    /// For any `Value` on which `is_boolean` returns true, [`as_bool`] is
    /// guaranteed to return the boolean value.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let value = Value::Bool(true);
    ///
    /// assert!(value.is_boolean());
    /// value.as_bool().unwrap();
    /// # }
    /// ```
    ///
    /// [`as_bool`]: #method.as_bool
    pub fn is_boolean(&self) -> bool {
        match *self {
            Value::Bool(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is null.
    ///
    /// For any `Value` on which `is_null` returns true, [`as_null`] is
    /// guaranteed to return `Some(())`.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let value = Value::Null;
    ///
    /// assert!(value.is_null());
    /// value.as_null().unwrap();
    /// # }
    /// ```
    ///
    /// [`as_null`]: #method.as_null
    pub fn is_null(&self) -> bool {
        match *self {
            Value::Null => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a number.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// assert!(Value::from(3.14).is_number());
    /// # }
    /// ```
    pub fn is_number(&self) -> bool {
        match *self {
            Value::Number(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is an object.
    ///
    /// For any `Value` on which `is_array` returns true, [`as_object`] and
    /// [`as_object_mut`] are guaranteed to return a reference to the map
    /// representing the object.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let mut value = Value::Object(Default::default());
    ///
    /// assert!(value.is_object());
    ///
    /// value.as_object().unwrap();
    /// value.as_object_mut().unwrap();
    /// # }
    /// ```
    ///
    /// [`as_object`]: #method.as_object
    /// [`as_object_mut`]: #method.as_object_mut
    pub fn is_object(&self) -> bool {
        match *self {
            Value::Object(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a string.
    ///
    /// For any `Value` on which `is_string` returns true, [`as_str`] is
    /// guaranteed to return the string slice.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let value = Value::String("Hello, world!".to_owned());
    ///
    /// assert!(value.is_string());
    /// value.as_str().unwrap();
    /// # }
    /// ```
    ///
    /// [`as_str`]: #method.as_str
    pub fn is_string(&self) -> bool {
        match *self {
            Value::String(_) => true,
            _ => false,
        }
    }

    /// Returns true if the `Value` is a number that can be represented as an
    /// `f64`.
    ///
    /// For any `Value` on which `is_f64` returns true, [`as_f64`] is
    /// guaranteed to return the floating point value.
    ///
    /// Currently this function returns true if and only if both [`is_i64`] and
    /// [`is_u64`] return false. This behavior is not a guarantee in the future.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let value = Value::from(3.14);
    ///
    /// assert!(value.is_f64());
    /// value.as_f64().unwrap();
    /// # }
    /// ```
    ///
    /// [`as_f64`]: #method.as_f64
    /// [`is_i64`]: #method.is_i64
    /// [`is_u64`]: #method.is_u64
    pub fn is_f64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_f64(),
            _ => false,
        }
    }

    /// Returns true if the `Value` is an integer between `i64::MIN` and
    /// `i64::MAX`.
    ///
    /// For any Value on which `is_i64` returns true, [`as_i64`] is guaranteed
    /// to return the integer value.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let pos = Value::from(3);
    /// let neg = Value::from(-3);
    ///
    /// assert!(pos.is_i64());
    /// assert!(neg.is_i64());
    ///
    /// pos.as_i64().unwrap();
    /// neg.as_i64().unwrap();
    /// # }
    /// ```
    ///
    /// [`as_i64`]: #method.as_i64
    pub fn is_i64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_i64(),
            _ => false,
        }
    }

    /// Returns true if the `Value` is an integer between `0` and `u64::MAX`.
    ///
    /// For any Value on which `is_u64` returns true, [`as_u64`] is guaranteed
    /// to return the integer value.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate json_api;
    /// #
    /// # use json_api::Value;
    /// #
    /// # fn main() {
    /// let value = Value::from(3);
    ///
    /// assert!(value.is_u64());
    /// value.as_u64().unwrap();
    /// # }
    /// ```
    ///
    /// [`as_u64`]: #method.as_u64
    pub fn is_u64(&self) -> bool {
        match *self {
            Value::Number(ref n) => n.is_u64(),
            _ => false,
        }
    }
}

/// Returns the `Value::Null`. This allows for better composition with `Option`
/// types.
///
/// # Example
///
/// ```
/// # extern crate json_api;
/// #
/// # use json_api::Value;
/// #
/// # fn main() {
/// const MSG: &'static str = "Hello, World!";
///
/// let opt = None;
/// let value = opt.map(Value::String).unwrap_or_default();
/// assert_eq!(value, Value::Null);
///
/// let opt = Some(MSG.to_owned());
/// let value = opt.map(Value::String).unwrap_or_default();
/// assert_eq!(value, Value::String(MSG.to_owned()));
/// # }
/// ```
impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl From<bool> for Value {
    fn from(inner: bool) -> Self {
        Value::Bool(inner)
    }
}

impl From<f32> for Value {
    fn from(n: f32) -> Self {
        Value::from(f64::from(n))
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Number::from_f64(n).map(Value::Number).unwrap_or_default()
    }
}

impl From<i8> for Value {
    fn from(n: i8) -> Self {
        Value::from(i64::from(n))
    }
}

impl From<i16> for Value {
    fn from(n: i16) -> Self {
        Value::from(i64::from(n))
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::from(i64::from(n))
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Number(n.into())
    }
}

impl From<u8> for Value {
    fn from(n: u8) -> Self {
        Value::from(u64::from(n))
    }
}

impl From<u16> for Value {
    fn from(n: u16) -> Self {
        Value::from(u64::from(n))
    }
}

impl From<u32> for Value {
    fn from(n: u32) -> Self {
        Value::from(u64::from(n))
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::Number(n.into())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<Map> for Value {
    fn from(data: Map) -> Self {
        Value::Object(data)
    }
}

impl<T> From<Option<T>> for Value
where
    T: Into<Value>,
{
    fn from(data: Option<T>) -> Self {
        data.map(T::into).unwrap_or_default()
    }
}

impl<T> From<Vec<T>> for Value
where
    T: Into<Value>,
{
    fn from(data: Vec<T>) -> Self {
        Value::Array(data.into_iter().map(|i| i.into()).collect())
    }
}

impl<'a> From<&'a str> for Value {
    fn from(s: &'a str) -> Self {
        Value::String(s.to_owned())
    }
}

impl<'a, T> From<&'a [T]> for Value
where
    T: Clone + Into<Value>,
{
    fn from(data: &'a [T]) -> Self {
        Value::Array(data.iter().cloned().map(|i| i.into()).collect())
    }
}

impl<T> FromIterator<T> for Value
where
    T: Into<Value>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        Value::Array(iter.into_iter().map(|i| i.into()).collect())
    }
}

impl FromIterator<(Key, Value)> for Value {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (Key, Value)>,
    {
        Value::Object(Map::from_iter(iter))
    }
}

impl FromStr for Value {
    type Err = ValueError;

    fn from_str(src: &str) -> Result<Self, Self::Err> {
        Ok(convert::from_json(src.parse()?)?)
    }
}

impl PartialEq<bool> for Value {
    fn eq(&self, rhs: &bool) -> bool {
        self.as_bool().map_or(false, |lhs| lhs == *rhs)
    }
}

impl PartialEq<f32> for Value {
    fn eq(&self, rhs: &f32) -> bool {
        *self == f64::from(*rhs)
    }
}

impl PartialEq<f64> for Value {
    fn eq(&self, rhs: &f64) -> bool {
        self.as_f64().map_or(false, |lhs| lhs == *rhs)
    }
}

impl PartialEq<i8> for Value {
    fn eq(&self, rhs: &i8) -> bool {
        *self == i64::from(*rhs)
    }
}

impl PartialEq<i16> for Value {
    fn eq(&self, rhs: &i16) -> bool {
        *self == i64::from(*rhs)
    }
}

impl PartialEq<i32> for Value {
    fn eq(&self, rhs: &i32) -> bool {
        *self == i64::from(*rhs)
    }
}

impl PartialEq<i64> for Value {
    fn eq(&self, rhs: &i64) -> bool {
        self.as_i64().map_or(false, |lhs| lhs == *rhs)
    }
}

impl PartialEq<isize> for Value {
    fn eq(&self, rhs: &isize) -> bool {
        *self == (*rhs as i64)
    }
}

impl PartialEq<u8> for Value {
    fn eq(&self, rhs: &u8) -> bool {
        *self == u64::from(*rhs)
    }
}

impl PartialEq<u16> for Value {
    fn eq(&self, rhs: &u16) -> bool {
        *self == u64::from(*rhs)
    }
}

impl PartialEq<u32> for Value {
    fn eq(&self, rhs: &u32) -> bool {
        *self == u64::from(*rhs)
    }
}

impl PartialEq<u64> for Value {
    fn eq(&self, rhs: &u64) -> bool {
        self.as_u64().map_or(false, |lhs| lhs == *rhs)
    }
}

impl PartialEq<usize> for Value {
    fn eq(&self, rhs: &usize) -> bool {
        *self == (*rhs as u64)
    }
}

impl PartialEq<str> for Value {
    fn eq(&self, rhs: &str) -> bool {
        self.as_str().map_or(false, |lhs| lhs == rhs)
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                f.write_str("any valid JSON API value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Value, E> {
                Ok(Value::Bool(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Value, E> {
                Ok(Value::from(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Value, E> {
                Ok(Value::Number(value.into()))
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Value, E> {
                self.visit_string(String::from(value))
            }

            fn visit_string<E>(self, value: String) -> Result<Value, E> {
                Ok(Value::String(value))
            }

            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut map = Map::with_capacity(access.size_hint().unwrap_or(0));

                while let Some(key) = access.next_key::<String>()? {
                    let key = key.parse().map_err(de::Error::custom)?;
                    let value = access.next_value()?;

                    map.insert(key, value);
                }

                Ok(Value::Object(map))
            }

            fn visit_seq<A>(self, mut access: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut array = Vec::with_capacity(access.size_hint().unwrap_or(0));

                while let Some(value) = access.next_element()? {
                    array.push(value);
                }

                Ok(Value::Array(array))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Value::Null => serializer.serialize_none(),
            Value::Array(ref value) => value.serialize(serializer),
            Value::Bool(value) => serializer.serialize_bool(value),
            Value::Number(ref value) => value.serialize(serializer),
            Value::Object(ref value) => value.serialize(serializer),
            Value::String(ref value) => serializer.serialize_str(value),
        }
    }
}

macro_rules! deserialize_number {
    ($method:ident) => {
        #[cfg(not(feature = "arbitrary_precision"))]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self {
                Value::Number(n) => Ok(n.deserialize_any(visitor)?),
                _ => Err(self.invalid_type(&visitor)),
            }
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self {
                Value::Number(n) => n.$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for Value {
    type Error = serde_json::Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Number(n) => Ok(n.deserialize_any(visitor)?),
            Value::String(v) => visitor.visit_string(v),
            Value::Array(v) => visit_array(v, visitor),
            Value::Object(v) => visit_object(v, visitor),
        }
    }

    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
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
        let (variant, value) = match self {
            Value::Object(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant.into(), Some(value))
            }
            Value::String(variant) => (variant, None),
            other => {
                return Err(serde::de::Error::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
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

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
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
        match self {
            Value::String(v) => visitor.visit_string(v),
            _ => Err(self.invalid_type(&visitor)),
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
        match self {
            Value::String(v) => visitor.visit_string(v),
            Value::Array(v) => visit_array(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Null => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Array(v) => visit_array(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Object(v) => visit_object(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Value::Array(v) => visit_array(v, visitor),
            Value::Object(v) => visit_object(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}


fn visit_array<'de, V>(array: Vec<Value>, visitor: V) -> Result<V::Value, serde_json::Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqDeserializer::new(array);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

fn visit_object<'de, V>(object: Map<Key, Value>, visitor: V) -> Result<V::Value, serde_json::Error>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapDeserializer::new(object);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

impl<'de> IntoDeserializer<'de, serde_json::Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<Value>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = serde_json::Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = serde_json::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Array(v)) => {
                serde::Deserializer::deserialize_any(SeqDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Object(v)) => {
                serde::Deserializer::deserialize_any(MapDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

pub(crate) struct SeqDeserializer<I>
    where I:Iterator<Item=Value>
{
    iter: I,
}

impl SeqDeserializer<std::vec::IntoIter<Value>> {
    fn new(vec: Vec<Value>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de, I> serde::Deserializer<'de> for SeqDeserializer<I>
    where I: Iterator<Item=Value> + ExactSizeIterator
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

impl<'de, I> SeqAccess<'de> for SeqDeserializer<I>
    where I: Iterator<Item=Value>,
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

struct MapDeserializer {
    iter: <Map<Key, Value> as IntoIterator>::IntoIter,
    value: Option<Value>,
}

impl MapDeserializer {
    fn new(map: Map<Key, Value>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = serde_json::Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::Owned(key.into()),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("map value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapDeserializer {
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

macro_rules! deserialize_value_ref_number {
    ($method:ident) => {
        #[cfg(not(feature = "arbitrary_precision"))]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match *self {
                Value::Number(ref n) => Ok(n.deserialize_any(visitor)?),
                _ => Err(self.invalid_type(&visitor)),
            }
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match *self {
                Value::Number(ref n) => n.$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    };
}

fn visit_array_ref<'de, V>(array: &'de [Value], visitor: V) -> Result<V::Value, serde_json::Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqRefDeserializer::new(array);
    let seq = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

fn visit_object_ref<'de, V>(object: &'de Map<Key, Value>, visitor: V) -> Result<V::Value, serde_json::Error>
where
    V: Visitor<'de>,
{
    let len = object.len();
    let mut deserializer = MapRefDeserializer::new(object);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

impl<'de> serde::Deserializer<'de> for &'de Value {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::Null => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(v),
            Value::Number(ref n) => Ok(n.deserialize_any(visitor)?),
            Value::String(ref v) => visitor.visit_borrowed_str(v),
            Value::Array(ref v) => visit_array_ref(v, visitor),
            Value::Object(ref v) => visit_object_ref(v, visitor),
        }
    }

    deserialize_value_ref_number!(deserialize_i8);
    deserialize_value_ref_number!(deserialize_i16);
    deserialize_value_ref_number!(deserialize_i32);
    deserialize_value_ref_number!(deserialize_i64);
    deserialize_value_ref_number!(deserialize_u8);
    deserialize_value_ref_number!(deserialize_u16);
    deserialize_value_ref_number!(deserialize_u32);
    deserialize_value_ref_number!(deserialize_u64);
    deserialize_value_ref_number!(deserialize_f32);
    deserialize_value_ref_number!(deserialize_f64);

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match *self {
            Value::Object(ref value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant.as_ref(), Some(value))
            }
            Value::String(ref variant) => (variant.as_ref(), None),
            ref other => {
                return Err(serde::de::Error::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumRefDeserializer { variant, value })
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

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::Bool(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::String(ref v) => visitor.visit_borrowed_str(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::String(ref v) => visitor.visit_borrowed_str(v),
            Value::Array(ref v) => visit_array_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::Null => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::Array(ref v) => visit_array_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::Object(ref v) => visit_object_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            Value::Array(ref v) => visit_array_ref(v, visitor),
            Value::Object(ref v) => visit_object_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct EnumRefDeserializer<'de> {
    variant: &'de str,
    value: Option<&'de Value>,
}

impl<'de> EnumAccess<'de> for EnumRefDeserializer<'de> {
    type Error = serde_json::Error;
    type Variant = VariantRefDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantRefDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantRefDeserializer<'de> {
    value: Option<&'de Value>,
}

impl<'de> VariantAccess<'de> for VariantRefDeserializer<'de> {
    type Error = serde_json::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(&Value::Array(ref v)) => {
                serde::Deserializer::deserialize_any(SeqRefDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(&Value::Object(ref v)) => {
                serde::Deserializer::deserialize_any(MapRefDeserializer::new(v), visitor)
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct SeqRefDeserializer<'de> {
    iter: std::slice::Iter<'de, Value>,
}

impl<'de> SeqRefDeserializer<'de> {
    fn new(slice: &'de [Value]) -> Self {
        SeqRefDeserializer { iter: slice.iter() }
    }
}

impl<'de> serde::Deserializer<'de> for SeqRefDeserializer<'de> {
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

impl<'de> SeqAccess<'de> for SeqRefDeserializer<'de> {
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

struct MapRefDeserializer<'de> {
    iter: <&'de Map<Key, Value> as IntoIterator>::IntoIter,
    value: Option<&'de Value>,
}

impl<'de> MapRefDeserializer<'de> {
    fn new(map: &'de Map<Key, Value>) -> Self {
        MapRefDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de> MapAccess<'de> for MapRefDeserializer<'de> {
    type Error = serde_json::Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                let key_de = MapKeyDeserializer {
                    key: Cow::Borrowed(&**key),
                };
                seed.deserialize(key_de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("map ref value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

impl<'de> serde::Deserializer<'de> for MapRefDeserializer<'de> {
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

struct MapKeyDeserializer<'de> {
    key: Cow<'de, str>,
}

macro_rules! deserialize_integer_key {
    ($method:ident => $visit:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match (self.key.parse(), self.key) {
                (Ok(integer), _) => visitor.$visit(integer),
                (Err(_), Cow::Borrowed(s)) => visitor.visit_borrowed_str(s),
                (Err(_), Cow::Owned(s)) => visitor.visit_string(s),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for MapKeyDeserializer<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        BorrowedCowStrDeserializer::new(self.key).deserialize_any(visitor)
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
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // Map keys cannot be null.
        visitor.visit_some(self)
    }

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
        self.key
            .into_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    forward_to_deserialize_any! {
        bool f32 f64 char str string bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl Value {
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected {
        match *self {
            Value::Null => Unexpected::Unit,
            Value::Bool(b) => Unexpected::Bool(b),
            Value::Number(_) => Unexpected::Other("number"),
            Value::String(ref s) => Unexpected::Str(s),
            Value::Array(_) => Unexpected::Seq,
            Value::Object(_) => Unexpected::Map,
        }
    }
}

struct BorrowedCowStrDeserializer<'de> {
    value: Cow<'de, str>,
}

impl<'de> BorrowedCowStrDeserializer<'de> {
    fn new(value: Cow<'de, str>) -> Self {
        BorrowedCowStrDeserializer { value }
    }
}

impl<'de> Deserializer<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Cow::Borrowed(string) => visitor.visit_borrowed_str(string),
            Cow::Owned(string) => visitor.visit_string(string),
        }
    }

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

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de> EnumAccess<'de> for BorrowedCowStrDeserializer<'de> {
    type Error = serde_json::Error;
    type Variant = UnitOnly;

    fn variant_seed<T>(self, seed: T) -> Result<(T::Value, Self::Variant), Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let value = seed.deserialize(self)?;
        Ok((value, UnitOnly))
    }
}

struct UnitOnly;

impl<'de> VariantAccess<'de> for UnitOnly {
    type Error = serde_json::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"newtype variant",
        ))
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"tuple variant",
        ))
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(de::Error::invalid_type(
            Unexpected::UnitVariant,
            &"struct variant",
        ))
    }
}