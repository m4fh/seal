use ureq::{self, Error as UreqError};
use mlua::prelude::*;

use crate::{std_io_colors as colors, std_json};
use crate::{table_helpers::TableBuilder, LuaValueResult};

pub fn net_get(luau: &Lua, get_config: LuaValue) -> LuaValueResult {
    match get_config {
        LuaValue::String(url) => {
            let url = url.to_str()?.to_string();
            match ureq::get(&url).call() {
                Ok(mut response) => {
                    let body = response.body_mut().read_to_string().into_lua_err()?;
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body.clone())?
                        .with_function("decode", {
                            move | luau: &Lua, _: LuaMultiValue | {
                                Ok(std_json::json_decode(luau, body.to_owned())?)
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(UreqError::StatusCode(code)) => {
                    let err_message = format!("HTTP error: {}", code);
                    let result = luau.create_table()?;
                    result.set("ok", false)?;
                    result.set("err", err_message)?;
                    Ok(LuaValue::Table(result))
                },
                Err(_) => {
                    wrap_err!("Some sort of HTTP I/O/transport/network error occurred...")
                }
            }
        },
        LuaValue::Table(_config) => {
            panic!("meow")
        }
        other => {
           wrap_err!("net.get expected url: string or GetOptions, got {:?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("get", net_get)?
        .build_readonly()
}
