use std::env;

use mlua::prelude::*;
use crate::table_helpers::TableBuilder as Table;

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {

	let formatted_os = match env::consts::OS {
		"linux" => String::from("Linux"),
		"windows" => String::from("Windows"),
		"android" => String::from("Android"),
		"macos" => String::from("MacOS"),
		_ => String::from("Other")
	};

	let mut executable_path = String::from("");
	let mut script_path = String::from("");

	let luau_args = {
		let rust_args: Vec<String> = env::args().collect();
		let result_args = luau.create_table()?;
		for (index, arg) in rust_args.iter().enumerate() {
			if index == 0 {
				executable_path = arg.to_string();
			} else if index == 1 {
				script_path = arg.to_string();
			} else {
				result_args.push(arg.to_string())?;
			}
		}
		result_args
	};

	let current_working_directory = env::current_dir()?.to_str().unwrap().to_string();

	Table::create(luau)?
		.with_value("os", formatted_os)?
		.with_value("args", luau_args)?
		.with_value("executable_path", executable_path)?
		.with_value("script_path", script_path)?
		.with_value("current_working_directory", current_working_directory)?
		.build_readonly()
}