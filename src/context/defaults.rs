use anyhow::Result;
use dynamic_function_macros::make_dyn;

use crate::models::Value;

#[make_dyn]
pub fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[make_dyn]
pub fn sub(a: u32, b: u32) -> u32 {
    a - b
}

#[make_dyn]
pub fn mul(a: u32, b: u32) -> u32 {
    a * b
}

#[make_dyn]
pub fn div(a: u32, b: u32) -> u32 {
    a / b
}

#[make_dyn]
pub fn modulus(a: u32, b: u32) -> u32 {
    a % b
}

pub fn println(args: &[Value]) -> Result<Value, String> {
    println!("{:?}", args);
    Ok(Value::Void)
}

pub fn throw(args: &[Value]) -> Result<Value, String> {
    Err(format!("{:?}", args))
}

#[make_dyn]
pub fn equals(arg0: Value, arg1: Value) -> bool {
    arg0 == arg1
}

#[make_dyn]
pub fn http_call(arg0: Value, arg1: Value) -> bool {
    arg0 == arg1
}

pub fn suppress(args: &[Value]) -> Result<Value, String> {
    Ok(Value::Void)
}
