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
        LuaValue::Table(config) => {
            let url: String = {
                match config.get("url")? {
                    LuaValue::String(url) => url.to_str()?.to_string(),
                    LuaValue::Nil => {
                        return wrap_err!("net.GetConfig missing field url")
                    },
                    other => {
                        return wrap_err!("net.get GetConfig expected url to be a string, got: {:?}", other)
                    }
                }
            };
            // let mut get_builder = ureq::get(&url);
            let mut get_builder = ureq::get(&url);

            if let LuaValue::Table(headers_table) = config.get("headers")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    get_builder = get_builder.header(key, value);
                }
            } else {};

            if let LuaValue::Table(headers_table) = config.get("params")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    get_builder = get_builder.query(key, value);
                }
            } else {};

            let body: Option<String> = {
                match config.get("body")? {
                    LuaValue::String(body) => Some(body.to_str()?.to_string()),
                    LuaValue::Table(body_table) => {
                        get_builder = get_builder.header("Content-Type", "application/json");
                        Some(std_json::json_encode(luau, LuaValue::Table(body_table))?)
                    },
                    LuaValue::Nil => None,
                    other => {
                        return wrap_err!("net.get GetOptions.body expected table (to serialize as json) or string, got: {:?}", other)
                    }
                }
            };

            let send_result = {
                if let Some(body) = body {
                    get_builder.force_send_body().send(body)
                } else {
                    get_builder.call()
                }
            };

            match send_result {
                Ok(mut result) => {
                    let body = result.body_mut().read_to_string().unwrap_or(String::from(""));
                    let json_decode_body = {
                        let body_clone = body.clone();
                        move |luau: &Lua, _: LuaMultiValue| {
                            match std_json::json_decode(luau, body_clone.to_owned()) {
                                Ok(response) => Ok(response),
                                Err(err) => {
                                    wrap_err!("NetResponse:decode() unable to decode response.body to json: {}", err)
                                }
                            }
                        }
                    };
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body)?
                        .with_function("decode", json_decode_body.to_owned())?
                        .with_function("unwrap", json_decode_body.to_owned())?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(err) => {
                    let err_result = TableBuilder::create(luau)?
                        .with_value("ok", false)?
                        .with_value("err", err.to_string())?
                        .with_function("unwrap", |_luau: &Lua, mut default: LuaMultiValue| {
                            let response = default.pop_front().unwrap();
                            let default = default.pop_back();
                            match default {
                                Some(LuaValue::Nil) => {
                                    wrap_err!("net.get: attempted to unwrap an erred request; note: default argument provided but was nil. Erred request: {:#?}", response)
                                },
                                None => {
                                    wrap_err!("net.get: attempted to unwrap an erred request without default argument. Erred request: {:#?}", response)
                                },
                                Some(other) => {
                                    Ok(other)
                                }
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(err_result))
                }
            }
        }
        other => {
           wrap_err!("net.get expected url: string or GetOptions, got {:?}", other)
        }
    }
}

pub fn net_post(luau: &Lua, get_config: LuaValue) -> LuaValueResult {
    match get_config {
        LuaValue::Table(config) => {
            let url: String = {
                match config.get("url")? {
                    LuaValue::String(url) => url.to_str()?.to_string(),
                    LuaValue::Nil => {
                        return wrap_err!("net.post: PostConfig missing url field")
                    },
                    other => {
                        return wrap_err!("net.post: PostConfig expected url to be a string, got: {:?}", other)
                    }
                }
            };
            // let mut get_builder = ureq::get(&url);
            let mut get_builder = ureq::post(&url);

            if let LuaValue::Table(headers_table) = config.get("headers")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    get_builder = get_builder.header(key, value);
                }
            } else {};

            if let LuaValue::Table(headers_table) = config.get("params")? {
                for pair in headers_table.pairs::<String, String>() {
                    let (key, value) = pair?;
                    get_builder = get_builder.query(key, value);
                }
            } else {};

            let body = {
                match config.get("body")? {
                    LuaValue::String(body) => body.to_str()?.to_string(),
                    LuaValue::Table(body_table) => {
                        get_builder = get_builder.header("Content-Type", "application/json");
                        std_json::json_encode(luau, LuaValue::Table(body_table))?
                    },
                    other => {
                        return wrap_err!("net.get GetOptions.body expected table (to serialize as json) or string, got: {:?}", other)
                    }
                }
            };

            let send_result = get_builder.send(body);

            match send_result {
                Ok(mut result) => {
                    let body = result.body_mut().read_to_string().unwrap_or(String::from(""));
                    let json_decode_body = {
                        let body_clone = body.clone();
                        move |luau: &Lua, _: LuaMultiValue| {
                            Ok(std_json::json_decode(luau, body_clone.to_owned())?)
                        }
                    };
                    let result = TableBuilder::create(luau)?
                        .with_value("ok", true)?
                        .with_value("body", body.clone())?
                        .with_function("decode", json_decode_body.to_owned())?
                        .with_function("unwrap", json_decode_body.to_owned())?
                        .build_readonly()?;
                    Ok(LuaValue::Table(result))
                },
                Err(err) => {
                    let err_result = TableBuilder::create(luau)?
                        .with_value("ok", false)?
                        .with_value("err", err.to_string())?
                        .with_function("unwrap", |_luau: &Lua, default: LuaValue| {
                            match default {
                                LuaValue::Nil => {
                                    wrap_err!("net.get: attempted to unwrap an erred request without default argument")
                                },
                                other => {
                                    Ok(other)
                                }
                            }
                        })?
                        .build_readonly()?;
                    Ok(LuaValue::Table(err_result))
                }
            }
        }
        other => {
           wrap_err!("net.post expected PostConfig, got {:?}", other)
        }
    }
}

fn net_request(_luau: &Lua, request_options: LuaValue) -> LuaValueResult {
    match request_options {
        LuaValue::Table(_options) => {
            todo!()
        },
        other => {
            wrap_err!("net.request expected table RequestOptions, got: {:?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("get", net_get)?
        .with_function("post", net_post)?
        .with_function("request", net_request)?
        .build_readonly()
}
