use mlua::prelude::*;
use std::{fs, env, process};

mod table_helpers;
mod output;
mod std_fs;
mod std_process;
mod std_env;
mod std_json;

fn require(luau: &Lua, path: String) -> LuaResult<LuaValue> {
    if path.starts_with("@std") {
        match path.as_str() {
            "@std/fs" => Ok(LuaValue::Table(std_fs::create(luau)?)),
            "@std/env" => Ok(LuaValue::Table(std_env::create(luau)?)),
            "@std/process" => Ok(LuaValue::Table(std_process::create(luau)?)),
            "@std/json" => Ok(LuaValue::Table(std_json::create(luau)?)),
            "@std/prettify" => Ok(LuaValue::Function(luau.create_function(output::prettify_output)?)),
            _ => {
                Err(LuaError::external(format!("program required an unexpected standard library: \"{}\"", &path)))
            }
        }
    } else if path.starts_with("@") {
        Err(LuaError::external("invalid require path or not impl yet"))
    } else if path.starts_with("./") {
        Err(LuaError::external("invalid require path or not impl yet"))
    } else {
        Err(LuaError::external("invalid require path or not impl yet"))
    }
}

fn main() -> LuaResult<()> {
    let luau = Lua::new();

    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("bad usage: did you forget to pass a file?");
        process::exit(1);
    }
    
    let luau_code: String = {
        let file_path = args[1].clone();
        if !file_path.ends_with(".luau") {
            eprintln!("file ext must be .luau");
            process::exit(1);
        } else if !fs::metadata(&file_path).is_ok() {
            eprintln!("Requested file doesn't exist: {}", &file_path);
            process::exit(1);
        } else {
            fs::read_to_string(&file_path)?
        }
    };

    luau.globals().set("require", luau.create_function(require)?)?;
    luau.globals().set("p", luau.create_function(output::debug_print)?)?;
    luau.globals().set("print", luau.create_function(output::pretty_print)?)?;

    let result = match luau.load(luau_code).exec() {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    };

    std_process::handle_exit_callback(&luau, 0)?;

    result

}
