use mlua::prelude::*;
use std::{fs, env, process, panic};

mod table_helpers;
mod output;
mod std_fs;
mod std_process;
mod std_env;
mod std_json;
mod std_time;
mod err_handling;

use crate::err_handling as errs;

type LuaValueResult = LuaResult<LuaValue>;

fn require(luau: &Lua, path: String) -> LuaValueResult {
    let table = LuaValue::Table;
    let function = LuaValue::Function;
    if path.starts_with("@std") {
        match path.as_str() {
            "@std/fs" => Ok(table(std_fs::create(luau)?)),
            "@std/env" => Ok(table(std_env::create(luau)?)),
            "@std/time" => Ok(table(std_time::create(luau)?)),
            "@std/process" => Ok(table(std_process::create(luau)?)),
            "@std/json" => Ok(table(std_json::create(luau)?)),
            "@std/prettify" => Ok(function(luau.create_function(output::prettify_output)?)),
            _ => {
                errs::wrap_with("program required an unexpected standard library: ", path)
            }
        }
    } else if path.starts_with("@") {
        todo!("require not impl yet")
        // Err(LuaError::external("invalid require path or not impl yet"))
    } else if path.starts_with("./") {
        todo!("require not impl yet")
    } else {
        errs::wrap_with(
            "Invalid require path: Luau requires must start with a require alias (ex. \"@alias/path.luau\") or relative path (ex. \"./path.luau\").", 
            "\nNotes:\n  - ending a require with .luau is optional\n  - implicit relative paths (ex. require(\"file.luau\") without ./) are no longer allowed; see: https://github.com/luau-lang/rfcs/pull/56"
        )
    }
}

fn main() -> LuaResult<()> {
    let luau = Lua::new();

    panic::set_hook(Box::new(|info| {
        let payload = info.payload().downcast_ref::<&str>().map(|s| s.to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "Unknown error, please report this to the manager (deviaze)".to_string());
        
        eprintln!("{}[ERR] {}{}", output::RED, payload, output::RESET);
    }));
    
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("Bad usage: did you forget to pass me a file?")
    }
    
    let luau_code: String = {
        let file_path = args[1].clone();

        if file_path.ends_with(".lua") {
            panic!("wrong language!! this runtime is meant for the Luau language, if you want to run .lua files, pick another runtime please.");
        } else if !file_path.ends_with(".luau") {
            panic!(r#"Invalid file extension: expected file path to end with .luau, got path: "{}{}{}"{}"#, output::RESET, &file_path, output::RED, output::RESET);
        } else if !fs::metadata(&file_path).is_ok() {
            panic!(r#"Requested file doesn't exist: "{}{}{}"{}"#, output::RESET, &file_path, output::RED, output::RESET);
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
