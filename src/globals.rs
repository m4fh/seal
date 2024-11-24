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
            "@std/io/format" => Ok(function(luau.create_function(std_io_output::prettify_output)?)),
            "@std/colors" => Ok(table(colors::create(luau)?)),

            "@std/time" => Ok(table(std_time::create(luau)?)),
			"@std/time/datetime" => Ok(table(std_time::create_datetime(luau)?)),

            "@std/process" => Ok(table(std_process::create(luau)?)),
			"@std/shellexec" => Ok(function(luau.create_function(std_shellexec::shellexec)?)),

            "@std/net" => Ok(table(std_net::create(luau)?)),
            "@std/json" => Ok(table(std_json::create(luau)?)),
			
			"@std/thread" => Ok(table(std_thread::create(luau)?)),
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
		wrap_err!(
			"Invalid require path: Luau requires must start with a require alias (ex. \"@alias/path.luau\") or relative path (ex. \"./path.luau\").".to_owned() +
            "\nNotes:\n  - ending a require with .luau is optional\n  - implicit relative paths (ex. require(\"file.luau\") without ./) are no longer allowed; see: https://github.com/luau-lang/rfcs/pull/56"
		)
    }
}

pub fn error(_luau: &Lua, error_value: LuaValue) -> LuaValueResult {
	wrap_err!("message: {:?}", error_value.to_string()?)
}

fn globals_try(luau: &Lua, f: LuaValue) -> LuaValueResult {
	match f {
		LuaValue::Function(f) => {
			let pcall: LuaFunction = luau.globals().get("pcall")?;
			let mut result  = pcall.call::<LuaMultiValue>(f)?;
			let success = result.pop_front().unwrap();
			let result = result.pop_front().unwrap_or(LuaNil);

			let result_table = luau.create_table()?;

			if let LuaValue::Boolean(success) = success {
				if success {
					result_table.set("ok", true)?;
					result_table.set("result", result)?;
					result_table.set("match", luau.create_function(
						|_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
							let self_table = match multivalue.pop_front().unwrap() {
								LuaValue::Table(self_table) => self_table,
								other => {
									return wrap_err!("TryResult:match() expected self to be a TryResult, got: {:?}", other);
								}
							};
							let handler = multivalue.pop_back().unwrap();
							match handler {
								LuaValue::Table(handler) => {
									let ok_result: LuaValue = self_table.get("result")?;
									let ok_handler: LuaValue = handler.get("ok")?;
									if let LuaValue::Function(ok_handler) = ok_handler {
										Ok(ok_handler.call::<LuaValue>(ok_result)?)
									} else {
										Ok(ok_handler)
									}
								},
								other => {
									wrap_err!("TryResult:match() expected handler to be a table with field 'ok', got {:?}", other)
								}
							}
						}
					)?)?;
					result_table.set("unwrap", luau.create_function(
						|_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
							match multivalue.pop_front() {
								Some(LuaValue::Table(self_table)) => {
									let result = self_table.get("result")?;
									Ok(result)
								},
								None => {
									wrap_err!("TryResult:unwrap() was called with zero arguments (not even self)")
								},
								other => {
									wrap_err!("TryResult:unwrap() wtf is self?: {:?}", other)
								}
							}
						}
					)?)?;
				} else {
					result_table.set("ok", false)?;
					result_table.set("err", result.clone())?;
					result_table.set("match", luau.create_function(
						|_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
							let self_table = match multivalue.pop_front().unwrap() {
								LuaValue::Table(self_table) => self_table,
								other => {
									return wrap_err!("TryResult:match() expected self to be self, got: {:?}", other);
								}
							};
							let handler = multivalue.pop_back().unwrap();
							match handler {
								LuaValue::Table(handler) => {
									let err_result: LuaValue = self_table.get("err")?;
									let err_handler: LuaValue = handler.get("err")?;
									if let LuaValue::Function(ok_handler) = err_handler {
										Ok(ok_handler.call::<LuaValue>(err_result)?)
									} else {
										Ok(err_handler)
									}
								},
								other => {
									wrap_err!("TryResult:match() expected handler to be a table with field 'err', got {:?}", other)
								}
							}
						}
					)?)?;
					result_table.set("unwrap", luau.create_function(
						|_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
							match multivalue.pop_front() {
								Some(LuaValue::Table(self_table)) => {
									let err_result: LuaError = self_table.get("err")?;
									match multivalue.pop_front() {
										Some(default_value) => {
											Ok(default_value)
										},
										None => {
											wrap_err!("Attempted to :unwrap() a TryResult without a default value! Error: {}", err_result.to_string())
										}
									}
								},
								None => {
									wrap_err!("TryResult:unwrap() was called with zero arguments (not even self)")
								},
								other => {
									wrap_err!("TryResult:unwrap() wtf is self?: {:?}", other)
								}
							}
						}
					)?)?;
				}
			} else {
				unreachable!("wtf else is success other than boolean??? {:?}", success);
			};

			Ok(LuaValue::Table(result_table))
		},
		other => {
			wrap_err!("try expected a function, got {:?}", other)
		}
	}
}

pub fn set_globals(luau: &Lua) -> LuaResult<LuaValue> {
	let globals = luau.globals();
	globals.set("require", luau.create_function(require)?)?;
	globals.set("error", luau.create_function(error)?)?;	
    globals.set("p", luau.create_function(std_io_output::debug_print)?)?;
    globals.set("pp", luau.create_function(std_io_output::pretty_print_and_return)?)?;
    globals.set("print", luau.create_function(std_io_output::pretty_print)?)?;
	globals.set("try", luau.create_function(globals_try)?)?;

	Ok(LuaNil)
}