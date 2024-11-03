use mlua::prelude::*;
use crate::table_helpers::TableBuilder;
use crate::{std_io_input, std_io_colors, std_io_output};


pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("input", std_io_input::create(luau)?)?
        .with_value("colors", std_io_colors::create(luau)?)?
        .with_value("output", std_io_output::create(luau)?)?
        .build_readonly()
}