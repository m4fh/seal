use std::{fs, time::Duration};
use std::sync::Mutex;
use std::thread;
#[allow(unused_imports)]
use std::sync::mpsc;

use std::sync::Arc;

use regex::Regex;

use crate::{table_helpers::TableBuilder, LuaValueResult, colors, globals_require};
use mlua::prelude::*;

fn thread_sleep(_luau: &Lua, duration: LuaNumber) -> LuaValueResult {
	let dur = Duration::from_millis(duration as u64);
	thread::sleep(dur);
	Ok(LuaNil)
}

fn thread_spawn(luau: &Lua, spawn_options: LuaValue) -> LuaValueResult {
	match spawn_options {
		LuaValue::Table(options) => {
			let mut thread_src_path = String::from("");
			let mut thread_called_from_path = String::from("");
			let spawn_src = {
				if let LuaValue::String(src) = options.get("src")? {
					let src = src.to_str()?.to_string();
					Ok(src)
				} else if let LuaValue::String(path) = options.get("path")? {
					let extract_path_re = Regex::new(r"^(.*[/\\])[^/\\]+\.luau$").unwrap();
					let script: LuaTable = luau.globals().get("script")?;
					let current_path: String = script.get("current_path")?;
					thread_called_from_path = current_path.to_owned();
					let captures = extract_path_re.captures(&current_path).unwrap();
					let new_path = &captures[1];

					let path = path.to_str()?.to_string();
					let path = path.replace("./", "");
					let path = format!("{new_path}{path}");
					thread_src_path = path.to_owned();
					Ok(fs::read_to_string(path).unwrap())
				} else {
					wrap_err!("thread.spawn expected table with fields src or path, got neither")
				}
			}?;
			let handle = thread::spawn(|| {
				let new_luau = mlua::Lua::new();

				globals_require::set_globals(&new_luau).unwrap();

				new_luau.globals().set("script",
					TableBuilder::create(&new_luau).unwrap()
						.with_value("current_path", thread_src_path).unwrap()
						.with_value("thread_parent_path", thread_called_from_path).unwrap()
						.with_value("src", spawn_src.to_owned()).unwrap()
						.build().unwrap()
				).unwrap();

				match new_luau.load(spawn_src).exec() {
					Ok(_) => {},
					Err(err) => {
						let replace_main_re = Regex::new(r#"\[string \"[^\"]+\"\]"#).unwrap();
						let globals = new_luau.globals();
						let script: LuaTable = globals.get("script").unwrap();
						let current_path: String = script.get("current_path").unwrap();
						let thread_parent_path: String = script.get("thread_parent_path").unwrap();
						let err_context: Option<String> = script.get("context").unwrap();
						let err_message = {
							let err_message = replace_main_re
								.replace_all(&err.to_string(), format!("[\"{}\"]", current_path))
								.replace("_G.error", "error")
								.to_string();
							if let Some(context) = err_context {
								let context = format!("{}[CONTEXT] {}{}{}\n", colors::BOLD_RED, context, colors::RESET, colors::RED);
								context + &err_message + &format!("\n THREAD CALLED FROM: {}", thread_parent_path)
							} else {
								err_message + &format!("\n{}THREAD CALLED FROM:{} [\"{}\"]", colors::BOLD_RED, colors::RESET, thread_parent_path)
							}
						};
						panic!("{}", err_message);
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
							match handle.join() {
								Ok(_) => {
									return Ok(LuaNil);
								},
								Err(_) => {
									return wrap_err!("error in called thread.spawn");
								}
							}
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
		.with_function("spawn", thread_spawn)?
		.with_function("sleep", thread_sleep)?
		.build_readonly()
}