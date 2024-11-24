use mlua::prelude::*;
use table_helpers::TableBuilder;
use std::{fs, env, panic, path::Path};
use regex::Regex;

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
mod std_thread;

mod globals;

use crate::std_io_colors as colors;

type LuaValueResult = LuaResult<LuaValue>;

fn main() -> LuaResult<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        panic!("Bad usage: did you forget to pass me a file?")
    }

    let first_arg = args[1].clone();

    if first_arg == "--help" || first_arg == "-h" {
        println!("help");
        return Ok(());
    }

    if args.len() == 3 && args[2] == "--debug" {
        // don't mess with panic formatting
    } else {
        panic::set_hook(Box::new(|info| {
            let payload = info.payload().downcast_ref::<&str>().map(|s| s.to_string())
                .or_else(|| info.payload().downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "Unknown error, please report this to the manager (deviaze)".to_string());
            
            eprintln!("{}[ERR]{}{} {}{}", colors::BOLD_RED, colors::RESET, colors::RED, payload, colors::RESET);
        }));
    }

    let luau: Lua = Lua::new();
    let globals = luau.globals();

    let mut luau_code: String = {
        if first_arg == "eval" {
            let table = LuaValue::Table;

            let globals = luau.globals();
            globals.set("fs", table(std_fs::create(&luau)?))?;
            globals.set("process", table(std_process::create(&luau)?))?;
            globals.set("net", table(std_net::create(&luau)?))?;

            globals.set("script", TableBuilder::create(&luau)?
                .with_value("current_path", "eval")?
                .build()?
            )?;

            if args.len() <= 2 {
                panic!("seal eval got nothing to eval, did you forget to pass in a string?");
            } else {
                args[2].clone()
            }
        } else {
            let file_path = first_arg.clone();

            if file_path.ends_with(".lua") {
                panic!("Wrong language! Pick a different runtime if you want to run Lua files.")
            }

            globals.set("script", TableBuilder::create(&luau)?
                .with_value("entry_path", file_path.to_owned())?
                .with_value("current_path", file_path.to_owned())?
                .with_value("required_files", luau.create_table()?)?
                .build()?
            )?;

            let path_metadata = fs::metadata(&file_path);
            match path_metadata {
                Ok(metadata) => {
                    if metadata.is_file() && file_path.ends_with(".luau") {
                        fs::read_to_string(&file_path)?
                    } else if metadata.is_dir() {
                        // we should be able to 'run' directories that contain an init.luau
                        let find_init_filepath = Path::new(&file_path).join("init.luau");
                        if find_init_filepath.exists() {
                            fs::read_to_string(&find_init_filepath)?
                        } else {
                            panic!(r#"seal: Requested file is actually a directory: "{}{}{}"{}{}"#, colors::RESET, &file_path, colors::RED, colors::RESET, "\n  Hint: add a file named 'init.luau' to run this directory itself :)");
                        }
                    } else {
                        panic!(r#"Invalid file extension: expected file path to end with .luau (or be a directory containing an init.luau), got path: "{}{}{}"{}"#, colors::RESET, &file_path, colors::RED, colors::RESET);
                    }
                },
                Err(err) => {
                    panic!("seal: Provided path is Not Ok: {}", err);
                }
            }
        }
    };
    
    // handle shebangs by stripping first line by slicing from first newline
    if luau_code.starts_with("#!") {
        if let Some(first_newline_pos) = luau_code.find('\n') {
            luau_code = luau_code[first_newline_pos + 1..].to_string();
        }
    }

    let script: LuaTable = globals.get("script")?;
    script.set("src", luau_code.to_owned())?;

    globals::set_globals(&luau)?;

    match luau.load(luau_code).exec() {
        Ok(()) => {
            std_process::handle_exit_callback(&luau, 0)?;
            Ok(())
        },
        Err(err) => {
            let replace_main_re = Regex::new(r#"\[string \"[^\"]+\"\]"#).unwrap();
            let script: LuaTable = globals.get("script")?;
            let current_path: String = script.get("current_path")?;
            let err_context: Option<String> = script.get("context")?;
            let err_message = {
                let err_message = replace_main_re
                    .replace_all(&err.to_string(), format!("[\"{}\"]", current_path))
                    .replace("_G.error", "error")
                    .to_string();
                if let Some(context) = err_context {
                    let context = format!("{}[CONTEXT] {}{}{}\n", colors::BOLD_RED, context, colors::RESET, colors::RED);
                    context + &err_message
                } else {
                    err_message
                }
            };
            panic!("{}", err_message);
        },
    }

}
