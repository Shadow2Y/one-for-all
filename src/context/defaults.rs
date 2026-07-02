use dynamic_function_macros::make_dyn;

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
