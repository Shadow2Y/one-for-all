use std::any::Any;
use std::collections::HashMap;
use std::process::Output;
use std::sync::Arc;

use serde::{Deserialize, Serialize, Serializer};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)] // Optional: Makes it serialize like clean JSON values
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Object(HashMap<String, Value>),
    #[serde(serialize_with = "serialize_native", skip_deserializing)]
    Native(Arc<dyn Any + Send + Sync>),
    Void,
}

// Helper to handle the non-serializable dyn Any
fn serialize_native<S>(_: &Arc<dyn Any + Send + Sync>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Option A: Serialize as a placeholder string (highly practical for debugging/logging)
    serializer.serialize_str("<native_object>")

    // Option B: Or, if you want serialization to fail when a native object is hit:
    // Err(serde::ser::Error::custom("Cannot serialize native objects"))
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Int(i64),
    Float(f64),
    String(String),
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Int(i) => write!(f, "{}", i),
            LiteralValue::String(s) => write!(f, "{}", s),
            LiteralValue::Float(fl) => write!(f, "{}", fl),
        }
    }
}

impl From<&LiteralValue> for Value {
    fn from(value: &LiteralValue) -> Self {
        match value {
            LiteralValue::Int(v) => Value::Int(*v),
            LiteralValue::Float(v) => Value::Float(*v),
            LiteralValue::String(v) => Value::String(v.clone()),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Object(o) => write!(f, "{:?}", o),
            Value::Native(_) => write!(f, "<Custom Dynamic Type>"),
            Value::Void => write!(f, ""),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Native(a), Value::Native(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self, String>;
}

impl Value {
    pub fn cast<T: FromValue>(&self) -> Result<T, String> {
        T::from_value(self)
    }
}

macro_rules! int_impl {
    ($($t:ty),*) => {
        $(
            impl FromValue for $t {
                fn from_value(value: &Value) -> Result<Self, String> {
                    match value {
                        Value::Int(v) => {
                            <$t>::try_from(*v)
                                .map_err(|_| format!("{} out of range", stringify!($t)))
                        }
                        _ => Err(format!("Expected {}", stringify!($t))),
                    }
                }
            }

            impl From<$t> for Value {
                fn from(value: $t) -> Self {
                    Value::Int(value as i64)
                }
            }
        )*
    };
}

int_impl!(i8, i16, i32, i64, isize, u8, u16, u32, usize);

impl FromValue for f64 {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Float(v) => Ok(*v),
            _ => Err("Expected float".into()),
        }
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl FromValue for bool {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Bool(v) => Ok(*v),
            _ => Err("Expected bool".into()),
        }
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::String(v) => Ok(v.clone()),
            _ => Err("Expected string".into()),
        }
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_owned())
    }
}

impl<T> FromValue for Arc<T>
where
    T: Any + Send + Sync + 'static,
{
    fn from_value(value: &Value) -> Result<Self, String> {
        match value {
            Value::Native(any) => {
                let cloned: Arc<dyn Any + Send + Sync> = any.clone();

                cloned.downcast::<T>().map_err(|_| "Type mismatch".into())
            }
            _ => Err("Expected custom object".into()),
        }
    }
}

impl<T> From<Arc<T>> for Value
where
    T: Any + Send + Sync + 'static,
{
    fn from(value: Arc<T>) -> Self {
        Value::Native(value)
    }
}

impl FromValue for Value {
    fn from_value(value: &Value) -> Result<Self, String> {
        Ok(value.clone())
    }
}

pub fn output_to_value(output: Output) -> Value {
    let mut map = HashMap::new();

    map.insert(
        "stdout".into(),
        Value::String(String::from_utf8_lossy(&output.stdout).into_owned()),
    );

    map.insert(
        "stderr".into(),
        Value::String(String::from_utf8_lossy(&output.stderr).into_owned()),
    );

    map.insert("success".into(), Value::Bool(output.status.success()));

    map.insert(
        "exit_code".into(),
        Value::Int(output.status.code().unwrap_or(-1) as i64),
    );

    Value::Object(map)
}
