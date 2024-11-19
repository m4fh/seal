#![allow(dead_code)]

use mlua::prelude::*;

type LuaValueResult = LuaResult<LuaValue>;

#[macro_export]
macro_rules! wrap_err {
    ($msg:expr) => {
        Err(LuaError::external(format!("{}{}{}", colors::RED, $msg, colors::RESET)))
    };
    ($msg:expr, $($arg:tt)*) => {
        Err(LuaError::external(format!("{}{}{}", colors::RED, format!($msg, $($arg)*), colors::RESET)))
    };
}

