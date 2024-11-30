use mlua::prelude::*;

use crate::{std_net_http, table_helpers::TableBuilder};

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_value("http", std_net_http::create(luau)?)?
        .build_readonly()
}
