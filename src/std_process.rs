use std::process::{self, Command};

use mlua::prelude::*;
use crate::{std_env, colors, table_helpers::TableBuilder, wrap_err, LuaValueResult};

fn process_run(luau: &Lua, run_options: LuaValue) -> LuaValueResult {
	match run_options {
		LuaValue::Table(run_options) => {
			match run_options.get("program")? {
				LuaValue::String(program_requested) => {
					let result_table = luau.create_table()?;
					let rust_args: Vec<String>  = {
						let luau_args: LuaValue = run_options.get("args")?;
						match luau_args {
							LuaValue::Table(luau_args) => {
								let mut rust_vec: Vec<String> = Vec::from_lua(LuaValue::Table(luau_args), luau)?;
								// passing untrimmed-whitespace strings directly into a program w/ Command::new usually breaks programs
								// so let's trim the whitespace just to make sure
								for s in rust_vec.iter_mut() {
									*s = s.trim().to_string();
								};
								rust_vec
							},
							LuaValue::Nil => {
								Vec::new()
							},
							_ => {
								return wrap_err!("expected RunOptions args to be {{string}} or nil, got {:?}", luau_args);
							}
						}
					};

					let output = {
						match run_options.get("shell")? {
							LuaValue::String(shell) => {
								let rust_shell_str = shell.to_str().unwrap().to_string();
								Command::new(rust_shell_str.clone())
									.arg(
										if rust_shell_str.as_str() == "pwsh" || rust_shell_str.as_str() == "powershell" {
											"-Command"
										} else {
											"-c"
										}
									)
									.arg(program_requested.to_string_lossy())
									.arg(rust_args.join(" "))
									.output()
									.expect("failed to execute process")
							},
							LuaValue::Nil => {
								Command::new(program_requested.to_string_lossy())
									.args(rust_args)
									.output()
									.expect("failed to execute process")
							},
							other => {
								return wrap_err!("expected RunOptions.shell to be either a string or nil, got {:?}", other);
							}
						}
						
					};

					let stderr = String::from_utf8_lossy(&output.stderr);
					let stdout = String::from_utf8_lossy(&output.stdout);

					result_table.set("stdout", stdout.clone())?;
					result_table.set("stderr", stderr.clone())?;
					if output.status.success() {
						result_table.set("ok", true)?;
						result_table.set("out", stdout)?;
						result_table.set("unwrap", luau.create_function(
							|_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
								let spawn_result = match multivalue.pop_front() {
									Some(LuaValue::Table(spawn_result)) => spawn_result,
									Some(LuaValue::Nil) => {
										return wrap_err!("ProcessRunResult:unwrap() expected self to be self, got nil");
									},
									Some(other) => {
										return wrap_err!("ProcessRunResult:unwrap() expected self to be self, got {:?}", other);
									}
									None => {
										return wrap_err!("ProcessRunResult:unwrap() expected self, got nothing. Did you forget a colon (:) (method syntax)?");
									}
								};
								let stdout: LuaValue = spawn_result.raw_get("stdout")?;
								Ok(stdout)
							})?
						)?;
					} else {
						result_table.set("ok", false)?;
						result_table.set("err", stderr)?;
						result_table.set("unwrap", luau.create_function(
							|_luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
								let _spawn_result = match multivalue.pop_front() {
									Some(LuaValue::Table(spawn_result)) => spawn_result,
									Some(LuaValue::Nil) => {
										return wrap_err!("ProcessRunResult:unwrap() expected self to be self, got nil");
									},
									Some(other) => {
										return wrap_err!("ProcessRunResult:unwrap() expected self to be self, got {:?}", other);
									}
									None => {
										return wrap_err!("ProcessRunResult:unwrap() expected self, got nothing. Did you forget a colon (:) (method syntax)?");
									}
								};
								match multivalue.pop_front() {
									Some(value) => Ok(value),
									None => {
										wrap_err!("Attempt to :unwrap() an erred process.run without a default value!")
									}
								}
							})?
						)?;
					}

					Ok(LuaValue::Table(result_table))
				}, 
				other => {
					wrap_err!("process.run expected table with field `program`, got: {:?}", other)
				}
			}
		},
		_ => {
			wrap_err!("process.run expected table of RunOptions ({{ program: string, args: {{ string }}?, etc. }}), got: {:?}", run_options)
		}
	}
	
}

fn process_shell(luau: &Lua, shell_command: LuaValue) -> LuaValueResult {
	let shell_path = std_env::get_current_shell();
	match shell_command {
		LuaValue::String(command) => {
			process_run(luau, LuaValue::Table(
				TableBuilder::create(luau)?
					.with_value("program", command)?
					.with_value("shell", shell_path)?
					.build_readonly()?
			))
		},
		other => {
			wrap_err!("process.shell(command) expected command to be a string, got: {:#?}", other)
		}
	}
}

fn set_exit_callback(luau: &Lua, f: Option<LuaValue>) -> LuaValueResult {
	if let Some(f) = f {
		match f {
			LuaValue::Function(f) => {
				let globals = luau.globals();
				globals.set("_process_exit_callback_function", f)?;
				Ok(LuaNil)
			}, 
			_ => {
				let err_message = format!("process.setexitcallback expected to be called with a function, got {:?}", f);
				Err(LuaError::external(err_message))
			}
		}
	} else {
		let err_message = format!("process.setexitcallback expected to be called with a function, got {:?}", f);
		Err(LuaError::external(err_message))
	}
}

pub fn handle_exit_callback(luau: &Lua, exit_code: i32) -> LuaResult<()> {
	match luau.globals().get("_process_exit_callback_function")? {
		LuaValue::Function(f) => {
			let _ = f.call::<i32>(exit_code);
		},
		LuaValue::Nil => {},
		_ => {
			unreachable!("what did you put into _process_exit_callback_function???");
		}
	}
	Ok(())
}

fn exit(luau: &Lua, exit_code: Option<LuaValue>) -> LuaResult<()> {
    let exit_code = if let Some(exit_code) = exit_code {
        match exit_code {
            LuaValue::Integer(i) => i,
            _ => {
                return wrap_err!("process.exit expected exit_code to be a number (integer) or nil, got {:?}", exit_code);
            }
        }
    } else {
        0
    };
	// if we have custom callback function let's call it 
	let globals = luau.globals();
	match globals.get("_process_exit_callback_function")? {
		LuaValue::Function(f) => {
			f.call::<i32>(exit_code)?;
		},
		LuaValue::Nil => {},
		other => {
			unreachable!("wtf is in _process_exit_callback_function other than a function or nil?: {:?}", other)
		}
	}
    process::exit(exit_code);
}


pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
	TableBuilder::create(luau)?
		.with_function("run", process_run)?
		.with_function("shell", process_shell)?
		.with_function("setexitcallback", set_exit_callback)?
        .with_function("exit", exit)?
		.build_readonly()
}
