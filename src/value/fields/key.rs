use std::borrow::Borrow;
use std::fmt::{self, Display, Formatter};
use std::ops::Deref;
use std::str::FromStr;

use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use thiserror::Error;

//use crate::error::Error;
use crate::sealed::Sealed;

#[derive(Error, Debug)]
#[error(r#"Could no parse key '{0}': {1}"#)]
pub struct ParseKeyError(String, String);

/// Represents a single member name.
///
/// When a new `Key` is parsed, the underlying value's casing convention is converted to
/// kebab-case.
///
/// # Example
///
/// ```
/// # extern crate json_api;
/// #
/// # use std::str::FromStr;
/// #
/// # use json_api::value::{Key, ParseKeyError};
/// #
/// # fn example() -> Result<(), ParseKeyError> {
/// let key = Key::from_str("someFieldName")?;
/// assert_eq!(key, "some-field-name");
/// #
/// # Ok(())
/// # }
/// #
/// # fn main() {
/// # example().unwrap()
/// # }
/// ```
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Key(String);

impl Key {
    #[doc(hidden)]
    #[inline]
    pub fn from_raw(value: String) -> Self {
        Key(value)
    }
}

impl AsRef<[u8]> for Key {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        self
    }
}

impl Borrow<str> for Key {
    fn borrow(&self) -> &str {
        self
    }
}

impl Deref for Key {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(self)
    }
}

impl From<Key> for String {
    fn from(key: Key) -> Self {
        let Key(value) = key;
        value
    }
}

impl FromStr for Key {
    type Err = ParseKeyError;

    fn from_str(source: &str) -> Result<Key, Self::Err> {
        if source.is_empty() {
            return Err(ParseKeyError(source.to_string(), "cannot be blank".to_string()));
        }

        // We should reserve a bit more than what we need so in
        // the event that we end up converting camelCase to
        // kebab-case, we don't have to reallocate.
        let mut dest = String::with_capacity(source.len() + 10);
        let mut chars = source.chars().peekable();

        while let Some(value) = chars.next() {
            match value {
                '\u{002e}'
                | '\u{002f}'
                | '\u{0040}'
                | '\u{0060}'
                | '\u{0000}'..='\u{001f}'
                | '\u{0021}'..='\u{0029}'
                | '\u{002a}'..='\u{002c}'
                | '\u{003a}'..='\u{003f}'
                | '\u{005b}'..='\u{005e}'
                | '\u{007b}'..='\u{007f}' => {
                    return Err(ParseKeyError(source.to_string(), format!("reserved '{}'", value)));
                }
                '_' | '-' | ' ' if dest.is_empty() => {
                    return Err(ParseKeyError(source.to_string(), format!("cannot start with '{}'", value)));
                }
                '_' | '-' | ' ' => match chars.peek() {
                    Some('-') | Some('_') | Some(' ') | Some('A'..='Z') => {
                        continue;
                    }
                    Some(_) => {
                        dest.push('-');
                    }
                    None => {
                        return Err(ParseKeyError(source.to_string(), format!("cannot end with '{}'", value)));
                    }
                },
                'A'..='Z' if dest.ends_with('-') => {
                    dest.push(as_lowercase(value));
                }
                'A'..='Z' => {
                    dest.push('-');
                    dest.push(as_lowercase(value));
                }
                _ => {
                    dest.push(value);
                }
            }
        }

        Ok(Key(dest))
    }
}

impl PartialEq<String> for Key {
    fn eq(&self, rhs: &String) -> bool {
        &self.0 == rhs
    }
}

impl PartialEq<str> for Key {
    fn eq(&self, rhs: &str) -> bool {
        &**self == rhs
    }
}

impl<'a> PartialEq<&'a str> for Key {
    fn eq(&self, rhs: &&str) -> bool {
        &**self == *rhs
    }
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D>(deserializer: D) -> Result<Key, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = Key;

            fn expecting(&self, f: &mut Formatter) -> fmt::Result {
                f.write_str("a valid json api member name")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                value.parse().map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(KeyVisitor)
    }
}

impl Serialize for Key {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self)
    }
}

impl Sealed for Key {}

#[inline]
fn as_lowercase(value: char) -> char {
    (value as u8 + 32) as char
}
