#![allow(dead_code)]
use std::fmt::Display;

use mlua::prelude::*;
use crate::output;

type LuaValueResult = LuaResult<LuaValue>;

pub fn wrap(message: &str) -> LuaValueResult {
    let err_message = format!("{}[ERR]\n{message}{}", output::RED, output::RESET);
    Err(LuaError::external(err_message))
}

pub fn wrap_with<T: Display>(message: &str, got: T) -> LuaValueResult {
    let err_message = format!("{}[ERR]\n  {message}{} {}", output::RED, output::RESET, got);
    Err(LuaError::external(err_message))
}

pub fn wrap_expected_got<T: Display>(message: &str, expected: &str, got: T) -> LuaValueResult {
    let err_message = format!("{}[ERR]\n  {message}{}\nExpected {expected}, got: {}", output::RED, output::RESET, got);
    Err(LuaError::external(err_message))
}
