use mlua::prelude::*;
use libc::{self, WIFEXITED, WEXITSTATUS};
use std::ffi::CString;

use crate::{table_helpers::TableBuilder, LuaValueResult, colors};

pub fn shellexec(luau: &Lua, value: LuaValue) -> LuaValueResult {
	match value {
		LuaValue::String(luau_command) => {
			let rust_command = luau_command.to_string_lossy();
			let c_command = match CString::new(rust_command) {
				Ok(c_command) => c_command,
				Err(err) => {
					return wrap_err!("shellexec: Error creating CString: {}", err);
				}
			};
			let status = unsafe { libc::system(c_command.as_ptr()) };

			let mut exit_status_code: Option<i32> = None;

			let ok = {
				if status == -1 {
					false
				} else {
					let exited = WIFEXITED(status);
					exit_status_code = if exited {
						Some(WEXITSTATUS(status))
					} else {
						Some(1)
					};
					matches!(exit_status_code, Some(0))
				}
			};

			Ok(LuaValue::Table(
				TableBuilder::create(luau)?
					.with_value("ok", ok)?
					.with_value("status_code", match exit_status_code {
						Some(exit_status_code) => exit_status_code,
						None => match ok {
							true => 0,
							false => 1,
						}
					})?
					.build_readonly()?
			))
		},
		other => {
			wrap_err!("shellexec(command) expected command to be a string, got: {:#?}", other)
		}
	}
}