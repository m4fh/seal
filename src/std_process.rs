use std::process::{self, Command};

use mlua::prelude::*;
use crate::{std_env, colors, table_helpers::TableBuilder, wrap_err, LuaValueResult};

struct RunOptions {
    program: String,
    args: Vec<String>,
    shell: Option<String>,
}

impl RunOptions {
    fn new(program: String, args: Vec<String>, shell: Option<String>) -> Self {
        RunOptions {
            program,
            args,
            shell
        }
    }

    fn from_table(luau: &Lua, run_options: LuaTable) -> LuaResult<Self> {
        let program = match run_options.raw_get("program").unwrap() {
            LuaValue::String(program) => {
                program.to_string_lossy()
            },
            LuaValue::Nil => {
                return wrap_err!("RunOptions missing field `program`; expected string, got nil");
            },
            other => {
                return wrap_err!("RunOptions.program expected to be a string, got: {:#?}", other);
            }
        };

        let args = match run_options.raw_get("args").unwrap() {
            LuaValue::Table(args) => {
                let mut rust_vec: Vec<String> = Vec::from_lua(LuaValue::Table(args), luau)?;
                // let's trim the whitespace just to make sure we pass valid args (untrimmed args might explode)
                for s in rust_vec.iter_mut() {
                    *s = s.trim().to_string();
                };
                rust_vec
            },
            LuaValue::Nil => {
                Vec::new()
            },
            other => {
                return wrap_err!("RunOptions.args expected to be {{string}} or nil, got: {:#?}", other);
            }
        };

        let shell = match run_options.raw_get("shell").unwrap() {
            LuaValue::String(shell) => {
                Some(shell.to_string_lossy())
            },
            LuaValue::Nil => {
                None
            },
            other => {
                return wrap_err!("RunOptions.shell expected to be a string or nil, got: {:#?}", other);
            }
        };

        Ok(RunOptions {
            program,
            args,
            shell
        })
        
    }
}

fn process_run(luau: &Lua, run_options: LuaValue) -> LuaValueResult {
    let options = match run_options {
        LuaValue::Table(run_options) => {
            RunOptions::from_table(luau, run_options)?
        },
        LuaValue::Nil => {
            return wrap_err!("process.run expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string? }}, got nil.");
        },
        other => {
            return wrap_err!("process.run expected RunOptions table of type {{ program: string, args: {{string}}?, shell: string? }}, got: {:#?}", other);
        }
    };

    let output = {
        if let Some(shell) = options.shell {
            Command::new(shell.clone())
                .arg(
                    if shell.as_str() == "pwsh" || shell.as_str() == "powershell" {
                        "-Command"
                    } else {
                        "-c"
                    }
                )
                .arg(options.program)
                .arg(options.args.join(" "))
                .output()
                .expect("process.run failed to execute process")
        } else {
            Command::new(options.program)
                .args(options.args)
                .output()
                .expect("process.run failed to execute process")
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if output.status.success() {
        let success_table = TableBuilder::create(luau)?
            .with_value("ok", true)?
            .with_value("out", stdout.clone())?
            .with_value("stdout", stdout.clone())?
            .with_value("stderr", stderr.clone())?
            .with_function("unwrap", {
                let stdout = stdout.clone().to_string();
                move | luau: &Lua, _multivalue: LuaMultiValue | -> LuaValueResult {
                    Ok(LuaValue::String(luau.create_string(stdout.clone())?))
                }
            })?
            .build_readonly()?;
        Ok(LuaValue::Table(success_table))
    } else {
        let failure_table = TableBuilder::create(luau)?
            .with_value("ok", false)?
            .with_value("err", stderr.clone())?
            .with_value("stdout", stdout.clone())?
            .with_value("stderr", stderr.clone())?
            .with_function("unwrap",
                | _luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    let _failure_table = multivalue.pop_front();
                    let default_arg = match multivalue.pop_front() {
                        Some(value) => value,
                        None => {
                            return wrap_err!("Attempt to ProcessRunResult:unwrap() an erred process.run without a default value!")
                        }
                    };
                    Ok(default_arg)
                }
            )?
            .build_readonly()?;
        Ok(LuaValue::Table(failure_table))
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
