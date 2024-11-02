use mlua::prelude::*;
use crate::table_helpers::TableBuilder;

fn json_encode(luau: &Lua, table: LuaValue) -> LuaResult<String> {
    match table {
        LuaValue::Table(t) => {
            Ok(serde_json::to_string(&t).map_err(LuaError::external)?)
        },
        other => {
            Err(LuaError::external(format!("json.encode expected any json-serializable table, got: {:?}", other)))
        }
    }
}

fn json_decode(luau: &Lua, json: String) -> LuaResult<LuaValue> {
    let json_result: serde_json::Value = serde_json::from_str(&json).map_err(LuaError::external)?;
    Ok(luau.to_value(&json_result)?)
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", json_encode)?
        .with_function("decode", json_decode)?
        .build_readonly()
}
