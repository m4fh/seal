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
            "@std/colors" => Ok(table(colors::create(luau)?)),
            "@std/time" => Ok(table(std_time::create(luau)?)),
            "@std/process" => Ok(table(std_process::create(luau)?)),
            "@std/net" => Ok(table(std_net::create(luau)?)),
            "@std/json" => Ok(table(std_json::create(luau)?)),
			"@std/thread" => Ok(table(std_thread::create(luau)?)),
            "@std/prettify" => Ok(function(luau.create_function(std_io_output::prettify_output)?)),
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

		let captures = extract_path_re.captures(&current_path).unwrap();
		let new_path = &captures[1];
		println!("{new_path}");
		let path = path.replace("./", "");
		let path = format!("{new_path}{path}");
		let path_ref = path.clone();

		let require_path = match fs::metadata(&path) {
			Ok(metadata) => {
				if metadata.is_file() {
					Ok(path)
				} else if metadata.is_dir() {
					let init_path = format!("{path}/init.luau");
					if fs::metadata(&init_path).is_ok() {
						Ok(init_path)
					} else {
						wrap_err!("require: required directory doesn't contain an init.luau")
					}
				} else {
					unreachable!("require: wtf is this path?")
				}
			}, 
			Err(_) => {
				wrap_err!("require: path \"{}\" doesn't exist", path)
			}
		}?;
		let data = fs::read_to_string(require_path)?;
		script.set("current_path", path_ref.to_owned())?;
		let result: LuaValue = luau.load(data).eval()?;
		script.set("current_path", current_path.to_owned())?;
		Ok(result)
    } else {
        errs::wrap_with(
            "Invalid require path: Luau requires must start with a require alias (ex. \"@alias/path.luau\") or relative path (ex. \"./path.luau\").", 
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