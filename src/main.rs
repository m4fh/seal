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
mod globals_require;

use crate::err_handling as errs;
use crate::std_io_colors as colors;

type LuaValueResult = LuaResult<LuaValue>;

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

    let globals = luau.globals();
    
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

        let script = table_helpers::TableBuilder::create(&luau)?
            .with_value("entry_path", file_path.to_owned())?
            .with_value("current_path", file_path.to_owned())?
            .with_value("required_files", luau.create_table()?)?
            .build()?;
        globals.set("script", script)?;
 
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

    let script: LuaTable = globals.get("script")?;
    script.set("src", luau_code.to_owned())?;

    globals.set("require", luau.create_function(globals_require::require)?)?;
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
