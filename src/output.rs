#![allow(dead_code)]

use regex::Regex;
use mlua::prelude::*;

pub const RESET: &str = "\x1b[0m";
pub const BLACK: &str = "\x1b[30m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";

pub const BRIGHT_BLACK: &str = "\x1b[90m";
pub const BRIGHT_RED: &str = "\x1b[91m";
pub const BRIGHT_GREEN: &str = "\x1b[92m";
pub const BRIGHT_YELLOW: &str = "\x1b[93m";
pub const BRIGHT_BLUE: &str = "\x1b[94m";
pub const BRIGHT_MAGENTA: &str = "\x1b[95m";
pub const BRIGHT_CYAN: &str = "\x1b[96m";
pub const BRIGHT_WHITE: &str = "\x1b[97m";

pub const BLACK_BG: &str = "\x1b[40m";
pub const RED_BG: &str = "\x1b[41m";
pub const GREEN_BG: &str = "\x1b[42m";
pub const YELLOW_BG: &str = "\x1b[43m";
pub const BLUE_BG: &str = "\x1b[44m";
pub const MAGENTA_BG: &str = "\x1b[45m";
pub const CYAN_BG: &str = "\x1b[46m";
pub const WHITE_BG: &str = "\x1b[47m";

pub const BRIGHT_BLACK_BG: &str = "\x1b[100m";
pub const BRIGHT_RED_BG: &str = "\x1b[101m";
pub const BRIGHT_GREEN_BG: &str = "\x1b[102m";
pub const BRIGHT_YELLOW_BG: &str = "\x1b[103m";
pub const BRIGHT_BLUE_BG: &str = "\x1b[104m";
pub const BRIGHT_MAGENTA_BG: &str = "\x1b[105m";
pub const BRIGHT_CYAN_BG: &str = "\x1b[106m";
pub const BRIGHT_WHITE_BG: &str = "\x1b[107m";

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
    Ok(luau.create_string(&result)?)
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
					format!("{standard_s}")
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

pub fn prettify_output(_: &Lua, value: LuaValue) -> LuaResult<String> {
	let mut result = String::from("");
	process_pretty_values(value, &mut result, 0)?;
	Ok(result)
}
