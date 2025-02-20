use std::fs;

use mlua::prelude::*;
use crate::{std_fs, table_helpers::TableBuilder, wrap_err, colors, LuaValueResult};

use serde_json_lenient as serde_json;

#[allow(dead_code)]
pub fn json_encode_old(_luau: &Lua, table: LuaValue) -> LuaResult<String> {
    match table {
        LuaValue::Table(t) => {
            Ok(serde_json::to_string_pretty(&t).map_err(LuaError::external)?)
        },
        other => {
            Err(LuaError::external(format!("json.encode expected any json-serializable table, got: {:?}", other)))
        }
    }
}

struct EncodeOptions {
    pretty: bool,
    sorted: bool,
}

pub fn json_encode(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaResult<String> {
    let table_to_encode = match multivalue.pop_front() {
        Some(LuaValue::Table(table)) => table,
        Some(other) => {
            return wrap_err!("json.encode expected the value to encode to be a table, got: {:#?}", other);
        }
        None => {
            return wrap_err!("json.encode expected a value to encode, got nothing");
        }
    };

    let encode_options = {
        let options_table = match multivalue.pop_front() {
            Some(LuaValue::Table(table)) => Some(table),
            Some(LuaNil) => None,
            Some(other) => {
                return wrap_err!("json.encode(value: any, options: EncodeOptions) expected options to be a table, got: {:#?}", other);
            },
            None => None,
        };
        if let Some(options_table) = options_table {
            let pretty = match options_table.raw_get::<LuaValue>("pretty")? {
                LuaValue::Boolean(pretty) => pretty,
                LuaNil => true,
                other => {
                    return wrap_err!("EncodeOptions.pretty expected to be a boolean or nil, got: {:#?}", other);
                }
            };
            let sorted = match options_table.raw_get::<LuaValue>("sorted")? {
                LuaValue::Boolean(ordered) => ordered,
                LuaNil => false,
                other => {
                    return wrap_err!("EncodeOptions.sorted expected to be a boolean or nil, got: {:#?}", other);
                },
            };

            EncodeOptions {
                pretty,
                sorted,
            }
        } else {
            EncodeOptions {
                pretty: true,
                sorted: false,
            }
        }
    };

    let temp_encoded = match serde_json::to_string(&table_to_encode) {
        Ok(t) => t,
        Err(err) => {
            return wrap_err!("json.encode: error encoding json: {}", err);
        }
    };
    let mut json_value = serde_json::from_str::<serde_json::Value>(&temp_encoded).unwrap();

    if encode_options.sorted {
        json_value.sort_all_objects();
    }

    if encode_options.pretty {
        let encoded = serde_json::to_string_pretty(&json_value).unwrap();
        Ok(encoded)
    } else {
        Ok(temp_encoded)
    }
}

pub fn json_encode_raw(_luau: &Lua, table: LuaValue) -> LuaResult<String> {
    let table_to_encode = match table {
        LuaValue::Table(t) => t,
        other => {
            return wrap_err!("json.encode_raw expected any json-serializable table, got: {:#?}", other);
        }
    };
    match serde_json::to_string(&table_to_encode) {
        Ok(t) => Ok(t),
        Err(err) => {
            wrap_err!("json.encode_raw: unable to encode table: {}", err)
        }
    }
}

fn parse_fix_numbers_rec(luau: &Lua, t: LuaTable) -> LuaValueResult {
    let tonumber: LuaFunction = luau.globals().get("tonumber")?;
    for pair in t.pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair?;
        match v {
            LuaValue::Table(v) => {
                let has_fixable_n: LuaValue = v.get("$serde_json_lenient::private::Number")?;
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

pub fn json_decode(luau: &Lua, json: String) -> LuaValueResult {
    let json_result: serde_json::Value = match serde_json::from_str(&json) {
        Ok(json) => json,
        Err(err) => {
            return wrap_err!("json: unable to decode json. serde_json error: {}", err.to_string());
        }
    };
    let luau_result = LuaTable::from_lua(luau.to_value(&json_result)?, luau)?;
    // unfortunately there seems to be a serde issue between mlua and serde_json that causes numbers to be incorrectly
    // decoded to { ["$serde_json::private::Number"] = "23" } or smth so we have to go thru and recursively fix all numbers manually
    let luau_result = parse_fix_numbers_rec(luau, luau_result)?;
    
    Ok(luau_result)
}

fn json_readfile(luau: &Lua, file_path: LuaValue) -> LuaValueResult {
    let file_content = std_fs::fs_readfile(luau, file_path)?;
    json_decode(luau, file_content.to_string()?)
}

fn json_writefile(luau: &Lua, json_write_options: LuaValue) -> LuaValueResult {
    match json_write_options {
        LuaValue::Table(options) => {
            let file_path: LuaValue = options.get("path")?;
            if file_path == LuaValue::Nil {
                wrap_err!("json.writefile expected JsonWritefileOptions.path, got nil")
            } else {
                let file_content = options.get("content")?;
                match file_content {
                    LuaValue::Table(content) => {
                        let json_result = json_encode(luau, LuaValue::Table(content).into_lua_multi(luau)?)?;
                        fs::write(file_path.to_string()?, json_result)?;
                        Ok(LuaNil)
                    },
                    LuaValue::String(json) => {
                        fs::write(file_path.to_string()?, json.to_str()?.to_string())?;
                        Ok(LuaNil)
                    },
                    LuaValue::Nil => {
                        wrap_err!("expected JsonWritefileOptions.content to be table (to encode to json) or string (already encoded json), got nil.")
                    },
                    other => wrap_err!("expected table (to encode to json and save) or string (of already encoded json), got: {:?}", other)
                }
            }
        },
        other => {
            wrap_err!("json.writefile expected JsonWritefileOptions, got: {:?}", other)
        }
    }
}

fn json_writefile_raw(luau: &Lua, json_write_options: LuaValue) -> LuaValueResult {
    match json_write_options {
        LuaValue::Table(options) => {
            let file_path: LuaValue = options.get("path")?;
            if file_path == LuaValue::Nil {
                wrap_err!("json.writefile_data expected JsonWritefileOptions.path, got nil")
            } else {
                let file_content = options.get("content")?;
                match file_content {
                    LuaValue::Table(content) => {
                        let json_result = json_encode_raw(luau, LuaValue::Table(content))?;
                        fs::write(file_path.to_string()?, json_result)?;
                        Ok(LuaNil)
                    },
                    LuaValue::String(json) => {
                        fs::write(file_path.to_string()?, json.to_str()?.to_string())?;
                        Ok(LuaNil)
                    },
                    LuaValue::Nil => {
                        wrap_err!("expected JsonWritefileOptions.content to be table (to encode to json) or string (already encoded json), got nil.")
                    },
                    other => wrap_err!("expected table (to encode to json and save) or string (of already encoded json), got: {:?}", other)
                }
            }
        },
        other => {
            wrap_err!("json.writefile expected JsonWritefileOptions, got: {:?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("encode", json_encode)?
        .with_function("encode_raw", json_encode_raw)?
        .with_function("decode", json_decode)?
        .with_function("readfile", json_readfile)?
        .with_function("writefile", json_writefile)?
        .with_function("writefile_raw", json_writefile_raw)?
        .build_readonly()
}
