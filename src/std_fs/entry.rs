use std::fs;
use std::io;
use std::path::PathBuf;
use mlua::prelude::*;
use crate::{std_time, require::ok_table};
use crate::{wrap_err, LuaValueResult, LuaEmptyResult, table_helpers::TableBuilder, colors};
use copy_dir::copy_dir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn get_path_from_entry(entry: LuaValue, function_name: &str) -> LuaResult<String> {
    match entry {
        LuaValue::Table(entry) => {
            match entry.raw_get("path")? {
                LuaValue::String(path) => Ok(path.to_string_lossy()),
                other => {
                    wrap_err!("{} expected self.path to be a string, got: {:#?}", function_name, other)
                },
            }
        },
        other => {
            wrap_err!("{} expected to be called with self (method call), got: {:#?}", function_name, other)
        }
    }
}

pub fn wrap_io_read_errors(err: std::io::Error, function_name: &str, file_path: &str) -> LuaValueResult {
    match err.kind() {
        io::ErrorKind::NotFound =>
            wrap_err!("{}: File not found: {}", function_name, file_path),
        io::ErrorKind::PermissionDenied =>
            wrap_err!("{}: Permission denied: {}", function_name, file_path),
        other => {
            wrap_err!("{}: Error reading file: {}", function_name, other)
        }
    }
}

pub fn wrap_io_read_errors_empty(err: std::io::Error, function_name: &str, file_path: &str) -> LuaEmptyResult {
    match err.kind() {
        io::ErrorKind::NotFound =>
            wrap_err!("{}: File not found: {}", function_name, file_path),
        io::ErrorKind::PermissionDenied =>
            wrap_err!("{}: Permission denied: {}", function_name, file_path),
        other => {
            wrap_err!("{}: Error reading file: {}", function_name, other)
        }
    }
}

pub fn metadata(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let entry_path = get_path_from_entry(value, "Entry:metadata()")?;
    let metadata = match fs::metadata(&entry_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors(err, "Entry:metadata()", &entry_path);
        }
    };
    let created_at = match metadata.created() {
        Ok(created_at) => {
            std_time::from_system_time(luau, created_at)?
        },
        Err(_err) => LuaNil,
    };
    let modified_at = match metadata.modified() {
        Ok(modified_at) => {
            std_time::from_system_time(luau, modified_at)?
        },
        Err(_err) => LuaNil,
    };
    let accessed_at = match metadata.accessed() {
        Ok(accessed_at) => {
            std_time::from_system_time(luau, accessed_at)?
        },
        Err(_err) => LuaNil,
    };

    let permissions = {
        let builder = TableBuilder::create(luau)?
            .with_value("readonly", metadata.permissions().readonly())?;
        if cfg!(unix) {
            let permissions_mode = metadata.permissions().mode();
            builder
                .with_value("unix_mode", permissions_mode)?
                .build_readonly()?
        } else {
            builder.build_readonly()?
        }
    };

    ok_table(TableBuilder::create(luau)?
        .with_value("created_at", created_at)?
        .with_value("modified_at", modified_at)?
        .with_value("accessed_at", accessed_at)?
        .with_value("permissions", permissions)?
        .build_readonly()
    )
}

pub fn copy_to(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let entry = match multivalue.pop_front() {
        Some(entry) => entry,
        None => {
            return wrap_err!("Entry:copy_to() expected to be called with self, was incorrectly called with zero arguments");
        }
    };
    let entry_path = get_path_from_entry(entry, "Entry:copy_to()")?;
    let destination_path = match multivalue.pop_front() {
        Some(LuaValue::String(value)) => value.to_string_lossy(),
        Some(other) => {
            return wrap_err!("Entry:copy_to(destination: string) expected destination to be a string, got: {:#?}", other);
        }
        None => {
            return wrap_err!("Entry:copy_to(destination: string) missing destination");
        }
    };

    let metadata = match fs::metadata(&entry_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors_empty(err, "Entry:copy_to()", &entry_path);
        }
    };

    let entry_to_destination_path = entry_path.clone() + " -> " + &destination_path;

    if metadata.is_dir() {
        match copy_dir(&entry_path, destination_path) {
            Ok(_unsuccessful) => {
                Ok(())
            },
            Err(err) => {
                wrap_io_read_errors_empty(err, "Entry:copy_to()", &entry_to_destination_path)
            }
        }
    } else {
        match fs::copy(&entry_path, destination_path) {
            Ok(_) => Ok(()),
            Err(err) => {
                wrap_io_read_errors_empty(err, "Entry:copy_to()", &entry_path)
            }
        }
    }
}

pub fn move_to(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let entry = match multivalue.pop_front() {
        Some(entry) => entry,
        None => {
            return wrap_err!("Entry:move_to() expected to be called with self, got nothing");
        }
    };
    let entry_path = get_path_from_entry(entry, "Entry:move_to()")?;
    let destination_path = match multivalue.pop_front() {
        Some(LuaValue::String(destination_path)) => destination_path.to_string_lossy(),
        Some(other) => {
            return wrap_err!("Entry:move_to(destination) expected destination to be a string, got: {:#?}", other);
        },
        None => {
            return wrap_err!("Entry:move_to(destination: string) was called without a destination path");
        }
    };

    let entry_to_destination_path = entry_path.clone() + "->" + &destination_path;

    match fs::rename(entry_path, destination_path) {
        Ok(_) => Ok(()),
        Err(err) => {
            wrap_io_read_errors_empty(err, "Entry:move_to()", &entry_to_destination_path)
        }
    }
}

pub fn rename(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let entry = match multivalue.pop_front() {
        Some(entry) => entry,
        None => {
            return wrap_err!("Entry:rename() called without self");
        }
    };
    let new_name = match multivalue.pop_front() {
        Some(LuaValue::String(value)) => {
            let name = value.to_string_lossy();
            if name.contains("/") {
                return wrap_err!("Entry:remove(): Entry name not allowed to contain '/'; use Entry:move_to(destination) to move a file");
            } else {
                name
            }
        },
        Some(other) => {
            return wrap_err!("Entry:rename(): expected new name to be a string, got: {:#?}", other);
        },
        None => {
            return wrap_err!("Entry:rename(): expected a new name, got nothing");
        }
    };
    let entry_path = get_path_from_entry(entry, "Entry:rename()")?;
    let entry_path = PathBuf::from(entry_path);
    let mut new_path = match entry_path.parent() {
        Some(parent) => parent.to_path_buf(),
        None => {
            if cfg!(target_os="windows") {
                return wrap_err!("Entry:rename(): renaming a top-level Entry is not allowed in Windows");
            } else {
                PathBuf::from("/")
            }
        }
    };
    new_path.push(new_name);
    match fs::rename(&entry_path, new_path) {
        Ok(_) => Ok(()),
        Err(err) => {
            wrap_err!("Entry:rename(): unable to rename '{}' due to err: {}", entry_path.display(), err)
        }
    }
}

pub fn remove(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let entry = match multivalue.pop_front() {
        Some(entry) => entry,
        None => {
            return wrap_err!("Entry:remove(path: string) expected to be called with self");
        }
    };
    let entry_path = get_path_from_entry(entry, "Entry:remove()")?;
    let entry_path = PathBuf::from(entry_path);
    if !entry_path.exists() {
        wrap_err!("Entry:remove(): attempt to remove nonexistent entry at '{}'; have you already removed it?", entry_path.display())
    } else if entry_path.is_dir() {
        match fs::remove_dir_all(&entry_path) {
            Ok(_) => Ok(()),
            Err(err) => {
                wrap_err!("Entry:remove(): unable to remove directory at '{}' due to err: {}", entry_path.display(), err)
            }
        }
    } else if entry_path.is_file() {
        match fs::remove_file(&entry_path) {
            Ok(_) => Ok(()),
            Err(err) => {
                wrap_err!("Entry:remove(): unable to remove file '{}' due to err: {}", entry_path.display(), err)
            }
        }
    } else {
        wrap_err!("Entry:remove(): attempt to remove an unexpected entry type at '{}'", entry_path.display())
    }
}