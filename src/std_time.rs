use std::{thread, time::Duration};
use crate::table_helpers::TableBuilder;
use mlua::prelude::*;

fn time_wait(_luau: &Lua, seconds: LuaNumber) -> LuaResult<LuaValue> {
    let millis = (seconds * 1000.0) as u64;
    let dur = Duration::from_millis(millis);
    thread::sleep(dur);
    Ok(LuaNil)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("wait", time_wait)?
        .build_readonly()
}