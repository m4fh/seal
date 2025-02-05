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
            "@std/fs/path" => Ok(table(std_fs::create_path(luau)?)),
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
            "@std/serde/hex" => Ok(table(std_serde::create_hex(luau)?)),
        
            "@std/net" => Ok(table(std_net::create(luau)?)),
            "@std/net/http" => Ok(table(std_net_http::create(luau)?)),
            "@std/net/http/server" => Ok(table(std_net_serve::create(luau)?)),
            "@std/net/request" => Ok(function(luau.create_function(std_net_http::http_request)?)),
            "@std/json" => Ok(table(std_json::create(luau)?)),

            "@std/crypt" => Ok(table(std_crypt::create(luau)?)),
            "@std/crypt/aes" => Ok(table(std_crypt::create_aes(luau)?)),
            "@std/crypt/rsa" => Ok(table(std_crypt::create_rsa(luau)?)),
            "@std/crypt/hash" => Ok(table(std_crypt::create_hash(luau)?)),
            "@std/crypt/password" => Ok(table(std_crypt::create_password(luau)?)),
        
            "@std/thread" => Ok(table(std_thread::create(luau)?)),

            "@std/testing" => Ok(table(std_testing::create(luau)?)),
            "@std/testing/try" => Ok(function(luau.create_function(std_testing::testing_try)?)),
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

        let require_cache: LuaTable = luau.globals().raw_get("_REQUIRE_CACHE").unwrap();
        let cached_result: Option<LuaValue> = require_cache.raw_get(require_path.clone())?;

        if let Some(cached_result) = cached_result {
            Ok(cached_result)
        } else {
            let data = fs::read_to_string(&require_path)?;
            script.raw_set("current_path", require_path.to_owned())?;
            let result: LuaValue = luau.load(data).set_name(&require_path).eval()?;
            require_cache.raw_set(require_path.clone(), result)?;
            // this is pretty cursed but let's just read the data we just wrote to the cache to get a new LuaValue
            // that can be returned without breaking the borrow checker or cloning
            let result = require_cache.raw_get(require_path.to_owned())?;
            script.raw_set("current_path", current_path.to_owned())?;
            Ok(result)
        }
    } else {
        wrap_err!(
            "Invalid require path: Luau requires must start with a require alias (ex. \"@alias/path\") or relative path (ex. \"./path\").".to_owned() +
            "\nNotes:\n  - ending a require path with .luau is no longer recommended in Luau but supported by seal\n  - implicit relative paths (ex. require(\"file.luau\") without ./) are no longer allowed; see: https://github.com/luau-lang/rfcs/pull/56"
        )
    }
}

pub fn error(_luau: &Lua, error_value: LuaValue) -> LuaValueResult {
    wrap_err!("message: {:?}", error_value.to_string()?)
}

pub fn warn(luau: &Lua, warn_value: LuaValue) -> LuaValueResult {
    let formatted_text = std_io_output::format_output(luau, warn_value)?;
    println!("{}{}{}", colors::BOLD_YELLOW, formatted_text, colors::RESET);
    Ok(LuaNil)
}

const SEAL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn set_globals(luau: &Lua) -> LuaValueResult {
    let globals: LuaTable = luau.globals();
    let luau_version: LuaString = globals.raw_get("_VERSION")?;
    globals.raw_set("require", luau.create_function(require)?)?;
    globals.raw_set("error", luau.create_function(error)?)?;	
    globals.raw_set("p", luau.create_function(std_io_output::debug_print)?)?;
    globals.raw_set("pp", luau.create_function(std_io_output::pretty_print_and_return)?)?;
    globals.raw_set("print", luau.create_function(std_io_output::pretty_print)?)?;
    globals.raw_set("warn", luau.create_function(warn)?)?;
    globals.raw_set("_VERSION", format!("seal {} | {}", SEAL_VERSION, luau_version.to_string_lossy()))?;
    globals.raw_set("_G", TableBuilder::create(luau)?
        // .with_metatable(TableBuilder::create(luau)?
        //     .with_value("__index", luau.globals())?
        //     .build_readonly()?
        // )?
        .build()?
    )?;
    globals.raw_set("_REQUIRE_CACHE", TableBuilder::create(luau)?.build()?)?;

    Ok(LuaNil)
}