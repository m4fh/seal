use std::fs;
use std::sync::Mutex;
use std::thread;
#[allow(unused_imports)]
use std::sync::mpsc;

use std::sync::Arc;

use regex::Regex;

use crate::{table_helpers::TableBuilder, LuaValueResult, colors, globals_require, std_io_output};
use mlua::prelude::*;

fn spawn(luau: &Lua, spawn_options: LuaValue) -> LuaValueResult {
	match spawn_options {
		LuaValue::Table(options) => {
			let spawn_src = {
				if let LuaValue::String(src) = options.get("src")? {
					let src = src.to_str()?.to_string();
					Ok(src)
				} else if let LuaValue::String(path) = options.get("path")? {
					let extract_path_re = Regex::new(r"^(.*[/\\])[^/\\]+\.luau$").unwrap();
					let script: LuaTable = luau.globals().get("script")?;
					let current_path: String = script.get("current_path")?;
					let captures = extract_path_re.captures(&current_path).unwrap();
					let new_path = &captures[1];

					let path = path.to_str()?.to_string();
					let path = path.replace("./", "");
					let path = format!("{new_path}{path}");
					Ok(fs::read_to_string(path).unwrap())
				} else {
					wrap_err!("thread.spawn expected table with fields src or path, got neither")
				}
			}?;
			let handle = thread::spawn(|| {
				let new_luau = mlua::Lua::new();
				let globals = new_luau.globals();
				globals.set("require", new_luau.create_function(globals_require::require).unwrap()).unwrap();
    			globals.set("p", new_luau.create_function(std_io_output::debug_print).unwrap()).unwrap();
				globals.set("pp", new_luau.create_function(std_io_output::pretty_print_and_return).unwrap()).unwrap();
				globals.set("print", new_luau.create_function(std_io_output::pretty_print).unwrap()).unwrap();

				match new_luau.load(spawn_src).exec() {
					Ok(_) => {},
					Err(err) => {
						eprintln!("{:?}", err);
					}
				}
			});
			// no clue why this works, got it off copilot but yay fearful concurrency :p 
			let arc_handle = Arc::new(Mutex::new(Some(handle)));
			Ok(LuaValue::Table(
				TableBuilder::create(luau)?
					.with_function("join", move |_luau: &Lua, _value: LuaValue|{
						let mut handle = arc_handle.lock().unwrap();
						if let Some(handle) = handle.take() {
							handle.join().unwrap();
						}
						Ok(LuaNil)
					})?
					.build_readonly()?
			))
		},
		other => {
			wrap_err!("thread.spawn: expected ThreadSpawnOptions table, got: {:?}", other)
		}
	}
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
	TableBuilder::create(luau)?
		.with_function("spawn", spawn)?
		.build_readonly()
}