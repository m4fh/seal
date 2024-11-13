use mlua::prelude::*;
use std::{fs, env, process, panic};

mod table_helpers;
mod std_io_output;
mod std_fs;
mod std_process;
mod std_env;
mod std_json;
mod std_time;
#[macro_use]
mod err_handling;
mod std_io;
mod std_io_colors;
mod std_io_input;
mod std_net;

use crate::err_handling as errs;
use crate::std_io_colors as colors;

type LuaValueResult = LuaResult<LuaValue>;

fn require(luau: &Lua, path: String) -> LuaValueResult {
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
            "@std/colors" => Ok(table(colors::create(luau)?)),
            "@std/time" => Ok(table(std_time::create(luau)?)),
            "@std/process" => Ok(table(std_process::create(luau)?)),
            "@std/net" => Ok(table(std_net::create(luau)?)),
            "@std/json" => Ok(table(std_json::create(luau)?)),
            "@std/prettify" => Ok(function(luau.create_function(std_io_output::prettify_output)?)),
            other => {
                wrap_err!("program required an unexpected standard library: {}", other)
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
        
        eprintln!("{}[ERR]{}{} {}{}", colors::BOLD_RED, colors::RESET, colors::RED, payload, colors::RESET);
    }));
    
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("Bad usage: did you forget to pass me a file?")
    }
    
    let luau_code: String =  'gotcoded: {
        let first_arg = args[1].clone();

        if first_arg == "eval" {
            let table = LuaValue::Table;
            let globals = luau.globals();
            globals.set("fs", table(std_fs::create(&luau)?))?;
            globals.set("process", table(std_process::create(&luau)?))?;
            globals.set("net", table(std_net::create(&luau)?))?;
            break 'gotcoded args[2].clone();
        };

        let file_path = first_arg;
 
        if file_path.ends_with(".lua") {
            panic!("wrong language!! this runtime is meant for the Luau language, if you want to run .lua files, pick another runtime please.");
        } else if !fs::metadata(&file_path).is_ok() {
            panic!(r#"Requested file doesn't exist: "{}{}{}"{}"#, colors::RESET, &file_path, colors::RED, colors::RESET);
        } else {
            if fs::metadata(&file_path)?.is_dir() {
                // we should be able to 'run' directories that contain a file named init.luau
                let find_init_filepath = String::from(file_path.clone() + "/init.luau");
                if fs::metadata(&find_init_filepath).is_ok() {
                    fs::read_to_string(&find_init_filepath)?
                } else {
                    panic!(r#"Requested file is actually a directory: "{}{}{}"{}{}"#, colors::RESET, &file_path, colors::RED, colors::RESET, "\n  Hint: add a file named 'init.luau' to run this directory itself :)");
                }
            } else if file_path.ends_with(".luau") {
                fs::read_to_string(&file_path)?
            } else {
                panic!(r#"Invalid file extension: expected file path to end with .luau (or be a directory containing an init.luau), got path: "{}{}{}"{}"#, colors::RESET, &file_path, colors::RED, colors::RESET);
            }
        }
    };

    let globals = luau.globals();

    globals.set("require", luau.create_function(require)?)?;
    globals.set("p", luau.create_function(std_io_output::debug_print)?)?;
    globals.set("pp", luau.create_function(std_io_output::pretty_print_and_return)?)?;
    globals.set("print", luau.create_function(std_io_output::pretty_print)?)?;

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
