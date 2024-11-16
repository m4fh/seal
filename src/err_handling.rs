#![allow(dead_code)]
use std::fmt::Display;

use mlua::prelude::*;
use crate::std_io_colors as colors;

type LuaValueResult = LuaResult<LuaValue>;

pub fn wrap(message: &str) -> LuaValueResult {
    let err_message = format!("{}[ERR]\n{message}{}", colors::RED, colors::RESET);
    Err(LuaError::external(err_message))
}

pub fn wrap_with<T: Display>(message: &str, got: T) -> LuaValueResult {
    let err_message = format!("{}[ERR]\n  {message}{} {}", colors::RED, colors::RESET, got);
    Err(LuaError::external(err_message))
}

pub fn wrap_expected_got<T: Display>(message: &str, expected: &str, got: T) -> LuaValueResult {
    let err_message = format!("{}[ERR]\n  {message}{}\nExpected {expected}, got: {}", colors::RED, colors::RESET, got);
    Err(LuaError::external(err_message))
}

#[macro_export]
macro_rules! wrap_err {
    ($msg:expr) => {
        Err(LuaError::external(format!("{}{}{}", colors::RED, $msg, colors::RESET)))
    };
    ($msg:expr, $($arg:tt)*) => {
        Err(LuaError::external(format!("{}{}{}", colors::RED, format!($msg, $($arg)*), colors::RESET)))
    };
}

