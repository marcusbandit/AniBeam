//! Words the core hands a shell to show (#792): a message ID from the catalog and its
//! arguments, never the sentence itself, so the shell renders them in whatever language
//! it speaks when it shows them, and a language switch reaches lines already said.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::i18n::{self, Arg};

/// One message: its catalog ID and the arguments it names. In JSON it is
/// `{"id": "scan-done", "args": {"count": 3, "folder": "Anime"}}`, which a shell passes
/// straight to its own lookup.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct Words {
    pub id: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub args: HashMap<String, WordValue>,
}

/// An argument: text, or a number, which also picks the plural form. In JSON it is the
/// bare number or string.
#[derive(Clone, Debug, PartialEq, uniffi::Enum)]
pub enum WordValue {
    Number { value: f64 },
    Text { value: String },
}

impl Serialize for WordValue {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            WordValue::Number { value } => s.serialize_f64(*value),
            WordValue::Text { value } => s.serialize_str(value),
        }
    }
}

impl<'de> Deserialize<'de> for WordValue {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Bare {
            Number(f64),
            Text(String),
        }
        Ok(match Bare::deserialize(d)? {
            Bare::Number(value) => WordValue::Number { value },
            Bare::Text(value) => WordValue::Text { value },
        })
    }
}

impl Words {
    pub fn new(id: impl Into<String>) -> Self {
        Words {
            id: id.into(),
            args: HashMap::new(),
        }
    }

    /// The same words with one more argument.
    pub fn arg(mut self, name: &str, value: impl Into<WordValue>) -> Self {
        self.args.insert(name.to_string(), value.into());
        self
    }

    /// The words in the language the core speaks now.
    pub fn render(&self) -> String {
        let args: Vec<(&str, Arg)> = self
            .args
            .iter()
            .map(|(name, value)| {
                let arg = match value {
                    WordValue::Number { value } => Arg::Num(*value),
                    WordValue::Text { value } => Arg::from(value.as_str()),
                };
                (name.as_str(), arg)
            })
            .collect();
        i18n::tr(&self.id, &args)
    }
}

impl std::fmt::Display for Words {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.render())
    }
}

impl From<&str> for WordValue {
    fn from(s: &str) -> Self {
        WordValue::Text {
            value: s.to_string(),
        }
    }
}

impl From<String> for WordValue {
    fn from(value: String) -> Self {
        WordValue::Text { value }
    }
}

impl From<&String> for WordValue {
    fn from(s: &String) -> Self {
        WordValue::Text { value: s.clone() }
    }
}

macro_rules! number_value {
    ($($t:ty),*) => {$(
        impl From<$t> for WordValue {
            fn from(n: $t) -> Self {
                WordValue::Number { value: n as f64 }
            }
        }
    )*};
}
number_value!(i32, i64, u32, u64, usize, f32, f64);

/// `words!("scan-done", count = n, folder = name)`: a [`Words`] with its arguments.
#[macro_export]
macro_rules! words {
    ($id:literal $(, $name:ident = $value:expr)* $(,)?) => {
        $crate::contract::Words::new($id)$(.arg(stringify!($name), $value))*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_travel_as_an_id_and_an_object_of_arguments() {
        let w = crate::words!("count-shows", count = 2u64, title = "Bebop");
        let json = serde_json::to_value(&w).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "id": "count-shows", "args": { "count": 2.0, "title": "Bebop" } })
        );
        let back: Words = serde_json::from_value(json).unwrap();
        assert_eq!(back, w);
        assert_eq!(
            serde_json::to_value(Words::new("x")).unwrap(),
            serde_json::json!({ "id": "x" })
        );
    }

    #[test]
    fn words_render_through_the_catalog() {
        assert_eq!(
            crate::i18n::tr_in(
                crate::i18n::Lang::En,
                "count-shows",
                &[("count", 2u64.into())]
            ),
            "2 shows"
        );
        assert_eq!(
            crate::words!("count-shows", count = 1u64).render(),
            "1 show"
        );
    }
}
