use std::fs;

use mlua::prelude::*;
use crate::{std_fs, table_helpers::TableBuilder};

pub fn json_encode(_luau: &Lua, table: LuaValue) -> LuaResult<String> {
    match table {
        LuaValue::Table(t) => {
            Ok(serde_json::to_string(&t).map_err(LuaError::external)?)
        },
        other => {
            Err(LuaError::external(format!("json.encode expected any json-serializable table, got: {:?}", other)))
        }
    }
}

fn parse_fix_numbers_rec(luau: &Lua, t: LuaTable) -> LuaResult<LuaValue> {
    let tonumber: LuaFunction = luau.globals().get("tonumber")?;
    for pair in t.pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        match v {
            LuaValue::Table(v) => {
                let has_fixable_n: LuaValue = v.get("$serde_json::private::Number")?;
                match has_fixable_n {
                    LuaValue::String(s) => {
                        let converted_n = tonumber.call::<LuaValue>(s)?;
                        match converted_n {
                            LuaValue::Integer(n) => {
                                t.set(k, n)?;
                            },
                            LuaValue::Number(n) => {
                                t.set(k, n)?;
                            },
                            _ => { unreachable!() }
                        }
                    },
                    LuaValue::Nil => {
                        parse_fix_numbers_rec(luau, v)?;
                    },
                    _ => {
                        unreachable!("Please don't use key `$serde_json::private::Number` for anything useful");
                    }
                }
            },
            _ => continue
        }
    }
    Ok(LuaValue::Table(t))
}

pub fn json_decode(luau: &Lua, json: String) -> LuaResult<LuaValue> {
    let json_result: serde_json::Value = serde_json::from_str(&json).map_err(LuaError::external)?;
    let luau_result = LuaTable::from_lua(luau.to_value(&json_result)?, luau)?;
    // unfortunately there seems to be a serde issue between mlua and serde_json that causes numbers to be incorrectly
    // decoded to { ["$serde_json::private::Number"] = "23" } or smth so we have to go thru and recursively fix all numbers manually
    let luau_result = parse_fix_numbers_rec(luau, luau_result)?;
    
    Ok(luau_result)
}

fn json_readfile(luau: &Lua, file_path: String) -> LuaResult<LuaValue> {
    let file_content = std_fs::fs_readfile(luau, file_path)?;
    Ok(json_decode(luau, file_content)?)
}

fn json_writefile(luau: &Lua, json_write_options: LuaValue) -> LuaResult<LuaValue> {
    match json_write_options {
        LuaValue::Table(options) => {
            let file_path: LuaValue = options.get("path")?;
            if file_path == LuaValue::Nil {
                Err(LuaError::external("expected JsonWritefileOptions.path, got nil"))
            } else {
                let file_content = options.get("content")?;
                match file_content {
                    LuaValue::Table(content) => {
                        let json_result = json_encode(luau, LuaValue::Table(content))?;
                        fs::write(file_path.to_string()?, json_result)?;
                        Ok(LuaNil)
                    },
                    LuaValue::String(json) => {
                        fs::write(file_path.to_string()?, json.to_str()?.to_string())?;
                        Ok(LuaNil)
                    },
                    LuaValue::Nil => {
                        Err(LuaError::external("expected JsonWritefileOptions.content to be table (to encode to json) or string (already encoded json), got nil."))
                    },
                    other => Err(LuaError::external(format!("expected table (to encode to json and save) or string (of already encoded json), got: {:?}", other)))
                }
            }
        },
        other => {
            let err_message = format!("json.writefile expected JsonWritefileOptions, got: {:?}", other);
            Err(LuaError::external(err_message))
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", json_encode)?
        .with_function("decode", json_decode)?
        .with_function("readfile", json_readfile)?
        .with_function("writefile", json_writefile)?
        .build_readonly()
}
