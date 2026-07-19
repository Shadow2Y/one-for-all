use anyhow::Result;

use crate::{context::registry::CommandRegistry, models::Value};

pub trait ValueProvider {
    fn get(&self, registry: &'static CommandRegistry, key: Option<&str>) -> Result<Value>;
}
