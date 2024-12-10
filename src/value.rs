use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::fmt::Formatter;
use toml::Value as TomlValue;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
/// The possible values we can get from the default or from a user
pub enum Value {
    String(String),
    Integer(i64),
    Boolean(bool),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Boolean(v) => write!(f, "{v}"),
            Value::String(v) => write!(f, "{v}"),
            Value::Integer(v) => write!(f, "{v}"),
        }
    }
}

impl Value {
    pub fn type_str(&self) -> &'static str {
        match self {
            Value::String(..) => "string",
            Value::Integer(..) => "integer",
            Value::Boolean(..) => "bool",
        }
    }

    pub(crate) fn is_str(&self) -> bool {
        matches!(self, Value::String(..))
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match *self {
            Value::String(ref s) => Some(&**s),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: TomlValue = Deserialize::deserialize(deserializer)?;
        match v {
            TomlValue::String(s) => Ok(Value::String(s)),
            TomlValue::Integer(i) => Ok(Value::Integer(i)),
            TomlValue::Boolean(b) => Ok(Value::Boolean(b)),
            _ => Err(D::Error::custom(format!("Value {} (of type `{}`) is not allowed as a value: only strings, integers and boolean are.", v, v.type_str()))),
        }
    }
}
