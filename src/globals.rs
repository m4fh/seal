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

            "@std/testing" => Ok(table(std_testing::create(luau)?)),
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
    } else if path.starts_with("./") || path.starts_with("../") {
        // TODO: unfuck this "path could not be extracted stuff"
        // regex should handle both windows and unix paths
        let extract_path_re = Regex::new(r"^(.*.*[/\\])[^/\\]+\.luau$").unwrap();
        let script: LuaTable = luau.globals().raw_get("script")?;
        let current_path: String = script.raw_get("current_path")?;

        let captures = match extract_path_re.captures(&current_path) {
            Some(captures) => captures,
            None => {
                return wrap_err!("require: path could not be extracted: {}", current_path);
            }
        };
        let new_path = &captures[1];
        let path = {
            if path.starts_with("./") {
                path.replace("./", "")
            } else {
                path
            }
        };
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

pub fn set_globals(luau: &Lua) -> LuaResult<LuaValue> {
    let globals = luau.globals();
    globals.set("require", luau.create_function(require)?)?;
    globals.set("error", luau.create_function(error)?)?;	
    globals.set("p", luau.create_function(std_io_output::debug_print)?)?;
    globals.set("pp", luau.create_function(std_io_output::pretty_print_and_return)?)?;
    globals.set("print", luau.create_function(std_io_output::pretty_print)?)?;

    Ok(LuaNil)
}