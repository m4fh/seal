use std::env;
use std::process::Command;

use mlua::prelude::*;
use crate::table_helpers::TableBuilder;

pub fn get_current_shell() -> String {
    #[cfg(target_family = "unix")]
    {
        // On Unix-like systems, check the SHELL environment variable
        if let Ok(shell_path) = env::var("SHELL") {
            return shell_path;
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows systems, first check the SHELL environment variable (if set)
        if let Ok(shell_path) = env::var("SHELL") {
            return shell_path;
        }
        
        // If SHELL is not set, check for PowerShell (pwsh.exe) or cmd (ComSpec)
        if let Ok(shell_path) = env::var("ComSpec") {
            return shell_path;
        }
        
        // Check specifically for PowerShell executables
        let pwsh_cmd = "pwsh";
        let powershell_cmd = "powershell";
        
        if let Ok(output) = Command::new("where").arg(pwsh_cmd).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return path;
            }
        }

        if let Ok(output) = Command::new("where").arg(powershell_cmd).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return path;
            }
        }
    }

    // As a fallback, try to find a shell using `which` or `where` command
    let which_cmd = if cfg!(target_family = "unix") {
        "which"
    } else if cfg!(target_os = "windows") {
        "where"
    } else {
        ""
    };

    if !which_cmd.is_empty() {
        if let Ok(output) = Command::new(which_cmd)
            .arg("sh") // You can replace "sh" with "bash" or "cmd" depending on what you want to check
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return path;
            }
        }
    }

    panic!("Could not determine the current shell path");
}

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

	TableBuilder::create(luau)?
		.with_value("os", formatted_os)?
		.with_value("args", luau_args)?
		.with_value("executable_path", executable_path)?
		.with_value("shell_path", get_current_shell())?
		.with_value("script_path", script_path)?
		.with_value("current_working_directory", current_working_directory)?
		.build_readonly()
}