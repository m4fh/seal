use std::process::{self, Command};

use mlua::prelude::*;
use crate::table_helpers::TableBuilder;

fn run_program(luau: &Lua, run_options: LuaValue) -> LuaResult<LuaTable> {
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
								panic!("expected SpawnOptions args to be {{string}} or nil, got {:?}", luau_args);
							}
						}
					};

					let output = {
						match run_options.get("shell")? {
							LuaValue::String(shell) => {
								let rust_shell_str = shell.to_str().unwrap().to_string();
								Command::new(rust_shell_str.clone())
									.arg(
										if rust_shell_str.as_str() == "powershell" {
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
								panic!("expected RunOptions.shell to be either a string or nil, got {:?}", other)
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
					} else {
						result_table.set("ok", false)?;
						result_table.set("err", stderr)?;
					}

					Ok(result_table)
				}, 
				_ => {
					Err(LuaError::external("process.spawn expected table with field `program`, got nil"))
				}
			}
		},
		_ => {
			let err_message = format!("process.spawn expected table of SpawnOptions ({{ program: string, args: {{ string }}?, etc. }}), got: {:?}", run_options);
			Err(LuaError::external(err_message))
		}
	}
	
}

fn exit(_luau: &Lua, exit_code: Option<LuaValue>) -> LuaResult<()> {
    let exit_code = if let Some(exit_code) = exit_code {
        match exit_code {
            LuaValue::Integer(i) => i as i32,
            _ => {
                panic!("process.exit expected exit_code to be a number (integer) or nil, got {:?}", exit_code);
            }
        }
    } else {
        0 as i32
    };
    process::exit(exit_code);
}


pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
	TableBuilder::create(luau)?
		.with_function("run", run_program)?
        .with_function("exit", exit)?
		.build_readonly()
}
