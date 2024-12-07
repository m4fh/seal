use std::fs;
use mlua::prelude::*;
use regex::Regex;
use crate::*;

pub fn require(luau: &Lua, path: String) -> LuaValueResult {
    let table = LuaValue::Table;
    let function = LuaValue::Function;
    if path.starts_with("@std") {
        match path.as_str() {
            "@std/fs" => Ok(table(std_fs::create(luau)?)),
            "@std/env" => Ok(table(std_env::create(luau)?)),
        
            "@std/io" => Ok(table(std_io::create(luau)?)),
            "@std/io/input" => Ok(table(std_io_input::create(luau)?)),
            "@std/io/output" => Ok(table(std_io_output::create(luau)?)),
            "@std/io/colors" => Ok(table(colors::create(luau)?)),
            "@std/io/clear" => Ok(function(luau.create_function(std_io_output::output_clear)?)),
            "@std/io/format" => Ok(function(luau.create_function(std_io_output::format_output)?)),
            "@std/colors" => Ok(table(colors::create(luau)?)),
        
            "@std/time" => Ok(table(std_time::create(luau)?)),
            "@std/time/datetime" => Ok(table(std_time::create_datetime(luau)?)),
        
            "@std/process" => Ok(table(std_process::create(luau)?)),
            "@std/shellexec" => Ok(function(luau.create_function(std_shellexec::shellexec)?)),
        
            "@std/serde" => Ok(table(std_serde::create(luau)?)),
            "@std/serde/base64" => Ok(table(std_serde::create_base64(luau)?)),
            "@std/serde/toml" => Ok(table(std_serde::create_toml(luau)?)),
            "@std/serde/yaml" => Ok(table(std_serde::create_yaml(luau)?)),
            "@std/serde/json" => Ok(table(std_json::create(luau)?)),
        
            "@std/net" => Ok(table(std_net::create(luau)?)),
            "@std/net/http" => Ok(table(std_net_http::create(luau)?)),
            "@std/net/http/server" => Ok(table(std_net_serve::create(luau)?)),
            "@std/net/request" => Ok(function(luau.create_function(std_net_http::http_request)?)),
            "@std/json" => Ok(table(std_json::create(luau)?)),

            "@std/crypt" => Ok(table(std_crypt::create(luau)?)),
            "@std/crypt/aes" => Ok(table(std_crypt::create_aes(luau)?)),
            "@std/crypt/rsa" => Ok(table(std_crypt::create_rsa(luau)?)),
        
            "@std/thread" => Ok(table(std_thread::create(luau)?)),
            "@std" => {
                Ok(table(
                    TableBuilder::create(luau)?
                        .with_value("fs", std_fs::create(luau)?)?
                        .with_value("env", std_env::create(luau)?)?
                        .with_value("io", std_io::create(luau)?)?
                        .with_value("colors", colors::create(luau)?)?
                        .with_value("format", function(luau.create_function(std_io_output::format_output)?))?
                        .with_value("time", std_time::create(luau)?)?
                        .with_value("datetime", std_time::create_datetime(luau)?)?
                        .with_value("process", std_process::create(luau)?)?
                        .with_value("shellexec", function(luau.create_function(std_shellexec::shellexec)?))?
                        .with_value("serde", std_serde::create(luau)?)?
                        .with_value("json", std_json::create(luau)?)?
                        .with_value("net", std_net::create(luau)?)?
                        .with_value("crypt", std_crypt::create(luau)?)?
                        .with_value("thread", std_thread::create(luau)?)?
                        .build_readonly()?
                ))
            }
            other => {
                wrap_err!("program required an unexpected standard library: {}", other)
            }
        }
    } else if path.starts_with("@") {
        todo!("require aliases not impl yet")
        // Err(LuaError::external("invalid require path or not impl yet"))
    } else if path.starts_with("./") {
        // regex should handle both windows and unix paths
        let extract_path_re = Regex::new(r"^(.*[/\\])[^/\\]+\.luau$").unwrap();
        let script: LuaTable = luau.globals().get("script")?;
        let current_path: String = script.get("current_path")?;

        let captures = match extract_path_re.captures(&current_path) {
            Some(captures) => captures,
            None => {
                return wrap_err!("require: path could not be extracted: {}", current_path);
            }
        };
        let new_path = &captures[1];
        let path = path.replace("./", "");
        let path = format!("{new_path}{path}");
        let path_ref = path.clone();

        let require_path = {
            let path = Path::new(&path);
            if path.exists() && path.is_file() {
                path.to_string_lossy().to_string()
            } else if path.exists() && path.is_dir() {
                let init_luau = path.join("init.luau");
                if init_luau.exists() && init_luau.is_file() {
                    init_luau.to_string_lossy().to_string()
                } else {
                    return wrap_err!("require: required directory doesn't contain an init.luau");
                }
            } else {
                let path_luau = path.to_string_lossy().to_string() + ".luau";
                let path_luau = Path::new(&path_luau);
                if path_luau.exists() && path_luau.is_file() {
                    path_luau.to_string_lossy().to_string()
                } else {
                    return wrap_err!("require: path {} doesn't exist", path_luau.to_string_lossy().to_string());
                }
            }
        };

        let data = fs::read_to_string(require_path)?;
        script.set("current_path", path_ref.to_owned())?;
        let result: LuaValue = luau.load(data).eval()?;
        script.set("current_path", current_path.to_owned())?;
        Ok(result)
    } else {
        wrap_err!(
            "Invalid require path: Luau requires must start with a require alias (ex. \"@alias/path.luau\") or relative path (ex. \"./path.luau\").".to_owned() +
            "\nNotes:\n  - ending a require with .luau is optional\n  - implicit relative paths (ex. require(\"file.luau\") without ./) are no longer allowed; see: https://github.com/luau-lang/rfcs/pull/56"
        )
    }
}

pub fn error(_luau: &Lua, error_value: LuaValue) -> LuaValueResult {
    wrap_err!("message: {:?}", error_value.to_string()?)
}

fn globals_try(luau: &Lua, f: LuaValue) -> LuaValueResult {
    match f {
        LuaValue::Function(f) => {
            let pcall: LuaFunction = luau.globals().get("pcall")?;
            let mut result  = pcall.call::<LuaMultiValue>(f)?;
            let success = result.pop_front().unwrap();
            let result = result.pop_front().unwrap_or(LuaNil);

            let result_table = luau.create_table()?;

            if let LuaValue::Boolean(success) = success {
                if success {
                    result_table.set("ok", true)?;
                    result_table.set("result", result)?;
                    result_table.set("match", luau.create_function(
                        |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            let self_table = match multivalue.pop_front().unwrap() {
                                LuaValue::Table(self_table) => self_table,
                                other => {
                                    return wrap_err!("TryResult:match() expected self to be a TryResult, got: {:?}", other);
                                }
                            };
                            let handler = multivalue.pop_back().unwrap();
                            match handler {
                                LuaValue::Table(handler) => {
                                    let ok_result: LuaValue = self_table.get("result")?;
                                    let ok_handler: LuaValue = handler.get("ok")?;
                                    if let LuaValue::Function(ok_handler) = ok_handler {
                                        Ok(ok_handler.call::<LuaValue>(ok_result)?)
                                    } else {
                                        Ok(ok_handler)
                                    }
                                },
                                other => {
                                    wrap_err!("TryResult:match() expected handler to be a table with field 'ok', got {:?}", other)
                                }
                            }
                        }
                    )?)?;
                    result_table.set("unwrap", luau.create_function(
                        |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            match multivalue.pop_front() {
                                Some(LuaValue::Table(self_table)) => {
                                    let result = self_table.get("result")?;
                                    Ok(result)
                                },
                                None => {
                                    wrap_err!("TryResult:unwrap() was called with zero arguments (not even self)")
                                },
                                other => {
                                    wrap_err!("TryResult:unwrap() wtf is self?: {:?}", other)
                                }
                            }
                        }
                    )?)?;
                    result_table.raw_set("expect_err", luau.create_function(
                        |_luau: &Lua, value: LuaValue| -> LuaValueResult {
                            wrap_err!("Error expected, but the function call didn't error. Instead, we got: {:#?}", value)
                        }
                    )?)?;
                } else {
                    result_table.set("ok", false)?;
                    result_table.set("err", result.clone())?;
                    result_table.set("match", luau.create_function(
                        |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            let self_table = match multivalue.pop_front().unwrap() {
                                LuaValue::Table(self_table) => self_table,
                                other => {
                                    return wrap_err!("TryResult:match() expected self to be self, got: {:?}", other);
                                }
                            };
                            let handler = multivalue.pop_back().unwrap();
                            match handler {
                                LuaValue::Table(handler) => {
                                    let err_result: LuaValue = self_table.get("err")?;
                                    let err_handler: LuaValue = handler.get("err")?;
                                    if let LuaValue::Function(ok_handler) = err_handler {
                                        Ok(ok_handler.call::<LuaValue>(err_result)?)
                                    } else {
                                        Ok(err_handler)
                                    }
                                },
                                other => {
                                    wrap_err!("TryResult:match() expected handler to be a table with field 'err', got {:?}", other)
                                }
                            }
                        }
                    )?)?;
                    result_table.set("unwrap", luau.create_function(
                        |_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            match multivalue.pop_front() {
                                Some(LuaValue::Table(self_table)) => {
                                    let err_result: LuaError = self_table.get("err")?;
                                    match multivalue.pop_front() {
                                        Some(default_value) => {
                                            Ok(default_value)
                                        },
                                        None => {
                                            wrap_err!("Attempted to :unwrap() a TryResult without a default value! Error: \n  {}", err_result.to_string())
                                        }
                                    }
                                },
                                None => {
                                    wrap_err!("TryResult:unwrap() was called with zero arguments (not even self)")
                                },
                                other => {
                                    wrap_err!("TryResult:unwrap() wtf is self?: {:?}", other)
                                }
                            }
                        }
                    )?)?;
                    result_table.set("expect_err", luau.create_function(
                        move |luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                            let _s = multivalue.pop_front();
                            let handler = match multivalue.pop_front() {
                                Some(handler) => handler,
                                None => LuaValue::Boolean(true),
                            };

                            match handler {
                                LuaValue::Function(handler) => {
                                    // call the handler function and return whatever it returns
                                    Ok(handler.call::<LuaValue>(result.clone())?)
                                },
                                LuaValue::String(error_matcher) => {
                                    // match error_matcher against the error we got using luau's string.match
                                    let string_lib: LuaTable = luau.globals().raw_get("string")?;
                                    let string_match: LuaFunction = string_lib.raw_get("match")?;

                                    // let global_tostring: LuaFunction = luau.globals().raw_get("tostring")?;
                                    // let stringified_err: LuaValue = global_tostring.call(result.clone())?;
                                    let stringified_err = std_io_output::strip_newlines_and_colors(&result.to_string()?);

                                    let match_args_vec: Vec<LuaValue> = vec!(
                                        luau.create_string(&stringified_err)?.into_lua(luau)?, 
                                        error_matcher.to_owned().into_lua(luau)?
                                    );
                            
                                    let match_args = LuaMultiValue::from_vec(match_args_vec);

                                    let does_match: bool = match string_match.call::<LuaValue>(match_args)? {
                                        LuaValue::String(_s) => true,
                                        _other => false,
                                    };

                                    if does_match {
                                        Ok(LuaValue::Boolean(true))
                                    } else {
                                        wrap_err!("Error did not match the expected error! Expected err to string.match {:#?}, got err:\n  {}", error_matcher, stringified_err)
                                    }
                                },
                                other => {
                                    wrap_err!("Expected error handler to be a function or a string (to string.match an error against), got: {:#?}", other)
                                }
                            }
                        }
                    )?)?;
                }
            } else {
                unreachable!("wtf else is success other than boolean??? {:?}", success);
            };

            Ok(LuaValue::Table(result_table))
        },
        other => {
            wrap_err!("try expected a function, got {:?}", other)
        }
    }
}

pub fn set_globals(luau: &Lua) -> LuaResult<LuaValue> {
    let globals = luau.globals();
    globals.set("require", luau.create_function(require)?)?;
    globals.set("error", luau.create_function(error)?)?;	
    globals.set("p", luau.create_function(std_io_output::debug_print)?)?;
    globals.set("pp", luau.create_function(std_io_output::pretty_print_and_return)?)?;
    globals.set("print", luau.create_function(std_io_output::pretty_print)?)?;
    globals.set("try", luau.create_function(globals_try)?)?;

    Ok(LuaNil)
}