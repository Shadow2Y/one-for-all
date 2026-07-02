use std::any::Any;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    // The escape hatch for any complex language-agnostic type
    Custom(Arc<dyn Any + Send + Sync>),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Custom(_) => write!(f, "<Custom Dynamic Type>"),
        }
    }
}
