use std::collections::HashMap;

use heck::*;
use tera::{to_value, try_get_value, Result, Tera, Value};

pub fn register_all_filters(tera: &mut Tera) {
    tera.register_filter("upper_camel_case", upper_camel_case);
    tera.register_filter("camel_case", camel_case);
    tera.register_filter("snake_case", snake_case);
    tera.register_filter("kebab_case", kebab_case);
    tera.register_filter("shouty_snake_case", shouty_snake_case);
    tera.register_filter("title_case", title_case);
    tera.register_filter("shouty_kebab_case", shouty_kebab_case);
}

pub fn upper_camel_case(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("upper_camel_case", "value", String, value);
    Ok(to_value(s.to_upper_camel_case()).unwrap())
}

pub fn camel_case(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("camel_case", "value", String, value);
    Ok(to_value(s.to_lower_camel_case()).unwrap())
}

pub fn snake_case(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("snake_case", "value", String, value);
    Ok(to_value(s.to_snake_case()).unwrap())
}

pub fn kebab_case(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("kebab_case", "value", String, value);
    Ok(to_value(s.to_kebab_case()).unwrap())
}

pub fn shouty_snake_case(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("shouty_snake_case", "value", String, value);
    Ok(to_value(s.to_shouty_snake_case()).unwrap())
}

pub fn title_case(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("title_case", "value", String, value);
    Ok(to_value(s.to_title_case()).unwrap())
}

pub fn shouty_kebab_case(value: &Value, _: &HashMap<String, Value>) -> Result<Value> {
    let s = try_get_value!("shouty_kebab_case", "value", String, value);
    Ok(to_value(s.to_shouty_kebab_case()).unwrap())
}
