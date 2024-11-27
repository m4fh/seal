#![allow(dead_code)]
#![allow(clippy::single_char_add_str)]

use regex::Regex;
use mlua::prelude::*;

use crate::{std_io_colors::*, table_helpers::TableBuilder, wrap_err, LuaValueResult};
use crate::std_io_colors as colors;

fn process_raw_values(value: LuaValue, result: &mut String, depth: usize) -> LuaResult<()> {
	let left_padding = " ".repeat(2 * depth);
	match value {
		LuaValue::Table(t) => {
			result.push_str("{\n");
			for pair in t.pairs::<LuaValue, LuaValue>() {
				let (k, v) = pair?;
				result.push_str(&format!("  {left_padding}{:?} = ", k));
				process_raw_values(v, result, depth + 1)?;
				result.push_str("\n");
			}
			result.push_str(&format!("{left_padding}}}"));
		},
		LuaValue::String(s) => {
			let formatted_string = format!("String({:?})", s);
			result.push_str(&formatted_string);
		},
		_ => {
			result.push_str(&format!("{:?}", value));
		}
	}
	if depth > 0 {
		result.push_str(",");
	}
	Ok(())
}

pub fn debug_print(luau: &Lua, stuff: LuaMultiValue) -> LuaResult<LuaString> {
    let mut result = String::from("");
    let mut multi_values = stuff.clone();

	while let Some(value) = multi_values.pop_front() {
		process_raw_values(value, &mut result, 0)?;
		if !multi_values.is_empty() {
			result += ", ";
		}
	}

    println!("{}", result.clone());
    luau.create_string(&result)
}

fn format_debug(luau: &Lua, stuff: LuaMultiValue) -> LuaResult<LuaString> {
	let mut result = String::from("");
    let mut multi_values = stuff.clone();

	while let Some(value) = multi_values.pop_front() {
		process_raw_values(value, &mut result, 0)?;
		if !multi_values.is_empty() {
			result += ", ";
		}
	}

    luau.create_string(&result)
}

fn process_pretty_values(value: LuaValue, result: &mut String, depth: usize) -> LuaResult<()> {
	let is_regular_identifier_re = Regex::new(r"^[A-Za-z_]+[0-9]*$").unwrap();
	let left_padding = " ".repeat(4 * depth);
	match value {
		LuaValue::Table(t) => {
			result.push_str("{\n");
			for pair in t.pairs::<LuaValue, LuaValue>() {
				let (k, v) = pair?;
				let formatted_k = match k {
					LuaValue::String(s) => {
						let standard_s = s.clone().to_string_lossy();
						if is_regular_identifier_re.is_match(&standard_s) {
							standard_s
						} else {
							format!("[{GREEN}\"{standard_s}\"{RESET}]")
						}
					},
					LuaValue::Number(n) => {
						let stringified_number = n.to_string();
						format!("{CYAN}[{RESET}{BLUE}{stringified_number}{RESET}{CYAN}]{RESET}")
					},
					LuaValue::Integer(n) => {
						let stringified_number = n.to_string();
						format!("{CYAN}[{RESET}{BLUE}{stringified_number}{RESET}{CYAN}]{RESET}")
					},
					LuaValue::Function(_f) => {
						format!("{RED}<function>{RESET}")
					},
					_ => {
						format!("{:?}", k)
					}
				};
				result.push_str(&format!("    {left_padding}{formatted_k} {CYAN}={RESET} "));
				process_pretty_values(v, result, depth + 1)?;
				result.push_str("\n");
			}
			result.push_str(&format!("{left_padding}}}"));
		},
		LuaValue::String(s) => {
			let standard_s = s.to_string_lossy();
			let formatted_s = {
				if depth == 0 {
					standard_s.to_string()
				} else {
					format!("{GREEN}\"{standard_s}\"{RESET}")
				}
			};
			result.push_str(&formatted_s);
		},
		LuaValue::Integer(i) => {
			let stringified_number = i.to_string();
			let formatted_number = format!("{CYAN}{stringified_number}{RESET}");
			result.push_str(&formatted_number);
		},
		LuaValue::Number(n) => {
			let stringified_number = n.to_string();
			let formatted_number = format!("{BLUE}{stringified_number}{RESET}");
			result.push_str(&formatted_number);
		},
		LuaValue::Function(f) => {
			let stringified_function = format!("{:?}", f);
			let formatted_function = format!("{RED}<{stringified_function}>{RESET}");
			result.push_str(&formatted_function);
		},
        LuaValue::Boolean(b) => {
            let formatted_bool = format!("{GREEN}{b}{RESET}");
            result.push_str(&formatted_bool);
        },
        LuaValue::Nil => {
            result.push_str(&format!("{RED}nil{RESET}"));
        },
		LuaValue::Error(err) => {
			let stringified_error = err.to_string();
			result.push_str(&format!("{RED}<{stringified_error}>{RESET}"));
		}
		_ => {
			result.push_str(&format!("{:?}", value));
		}
	}
	if depth > 0 {
		result.push_str(format!("{CYAN},{RESET}").as_str());
	}
	Ok(())
}


pub fn pretty_print(_: &Lua, values: LuaMultiValue) -> LuaResult<()> {
	let mut result = String::from("");
    let mut multi_values = values.clone();

	while let Some(value) = multi_values.pop_front() {
		process_pretty_values(value, &mut result, 0)?;
		if !multi_values.is_empty() {
			result += ", ";
		}
	}

    println!("{}", result.clone());
	Ok(())
}

pub fn pretty_print_and_return(_: &Lua, values: LuaMultiValue) -> LuaResult<String> {
	let mut result = String::from("");
    let mut multi_values = values.clone();

	while let Some(value) = multi_values.pop_front() {
		process_pretty_values(value, &mut result, 0)?;
		if !multi_values.is_empty() {
			result += ", ";
		}
	}

    println!("{}", result.clone());
	Ok(result)
}

pub fn prettify_output(_: &Lua, value: LuaValue) -> LuaResult<String> {
	let mut result = String::from("");
	process_pretty_values(value, &mut result, 0)?;
	Ok(result)
}

pub fn strip_newlines_and_colors(input: &str) -> String {
    let re_colors = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
    let without_colors = re_colors.replace_all(input, "");
    without_colors.to_string()
}

fn output_unformat(luau: &Lua, value: LuaValue) -> LuaValueResult {
	let input = match value {
		LuaValue::String(i) => i.to_string_lossy(),
		other => {
			return wrap_err!("expected string to strip formatting of, got: {:#?}", other)
		}
	};
	Ok(LuaValue::String(
		luau.create_string(input.as_str())?
	))
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
	TableBuilder::create(luau)?
		.with_function("format", prettify_output)?
		.with_function("unformat", output_unformat)?
		.with_function("print-and-return", pretty_print_and_return)?
		.with_function("debug-print", debug_print)?
		.with_function("debug-format", format_debug)?
		.build_readonly()
}
