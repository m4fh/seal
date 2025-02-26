use crate::{wrap_err, LuaValueResult, LuaEmptyResult, table_helpers::TableBuilder, colors};
use crate::{std_fs, std_time, require::ok_table};
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::{BufReader, Read, BufRead};
use std::io::Write;
use std::path::PathBuf;
use copy_dir::copy_dir;
use mlua::prelude::*;
use std::fs;
use std::io;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn get_path_from_entry(entry: LuaValue, function_name: &str) -> LuaResult<String> {
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

fn wrap_io_read_errors(err: std::io::Error, function_name: &str, file_path: &str) -> LuaValueResult {
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

fn wrap_io_read_errors_empty(err: std::io::Error, function_name: &str, file_path: &str) -> LuaEmptyResult {
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

fn entry_readfile(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = get_path_from_entry(value, "FileEntry:read()")?;
    let bytes = match fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return wrap_io_read_errors(err,"FileEntry:read()", &file_path);
        }
    };
    Ok(LuaValue::String(luau.create_string(bytes)?))
}

fn entry_readbytes(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let entry = match multivalue.pop_front() {
        Some(value) => value,
        None => {
            return wrap_err!("FileEntry:readbytes() incorrectly called with zero arguments");
        }
    };
    let entry_path = get_path_from_entry(entry, "FileEntry:readbytes()")?;

    std_fs::read_entry_path_into_buffer(luau, entry_path, multivalue, "FileEntry:readbytes")
}

fn entry_filesize(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = get_path_from_entry(value, "FileEntry:size()")?;
    let metadata = match fs::metadata(&file_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors(err, "FileEntry:size()", &file_path);
        }
    };
    Ok(LuaValue::Number(metadata.len() as f64))
}

fn entry_append(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let entry = match multivalue.pop_front() {
        Some(value) => value,
        None => {
            return wrap_err!("FileEntry:append(content) expected to be called with self but was incorrectly called with zero arguments");
        }
    };

    let entry_path = get_path_from_entry(entry, "FileEntry:append(content: string | buffer)")?;

    let mut file = match OpenOptions::new()
        .append(true)
        .open(&entry_path) 
    {
        Ok(file) => file,
        Err(err) => {
            return wrap_io_read_errors_empty(err, "FileEntry:append", &entry_path);
        }
    };
    
    let content = match multivalue.pop_front() {
        Some(LuaValue::String(content)) => {
            let content = content.to_string_lossy();
            content.as_bytes().to_owned()
        },
        Some(LuaValue::Buffer(buffy)) => {
            buffy.to_vec()
        },
        Some(other) => {
            return wrap_err!("FileEntry:append(content) expected content to be a string or buffer, got: {:#?}", other);
        },
        None => {
            return wrap_err!("FileEntry:append(content) expected arguments self and content but got no second argument");
        }
    };

    match file.write_all(&content) {
        Ok(_) => Ok(()),
        Err(err) => {
            wrap_err!("FileEntry:append: error writing to file: {}", err)
        }
    }

}

// TODO: investigate whether this is an actually good way of iterating thru lines or whether this is cursed
// something tells me this isn't as performant as it can be
// we can't make this thing return FnMut due to mlua reasons so we have to keep reader and current_line in refcells
pub fn readlines(luau: &Lua, entry_path: &str, function_name: &str) -> LuaValueResult {
    let file = match fs::File::open(entry_path) {
        Ok(file) => file,
        Err(err) => {
            return wrap_io_read_errors(err, function_name, entry_path);
        }
    };

    let function_name = function_name.to_owned();

    let reader = BufReader::new(file);
    let reader_cell = RefCell::new(reader);

    let current_line = 0;
    let current_line_cell = RefCell::new(current_line);

    Ok(LuaValue::Function(luau.create_function({
        move | luau: &Lua, _value: LuaValue | -> LuaResult<LuaMultiValue> {
            let mut reader_cell = reader_cell.borrow_mut();
            let reader = reader_cell.by_ref();
            let mut new_line = String::new();
            match reader.read_line(&mut new_line) {
                Ok(0) => {
                    let multi_vec = vec![LuaNil];
                    Ok(LuaMultiValue::from_vec(multi_vec))
                },
                Ok(_other) => {
                    let mut current_line = current_line_cell.borrow_mut();
                    *current_line += 1;
                    let luau_line = luau.create_string(new_line.trim_end())?;
                    let multi_vec = vec![LuaValue::Integer(*current_line), LuaValue::String(luau_line)];
                    Ok(LuaMultiValue::from_vec(multi_vec))
                },
                Err(err) => {
                    wrap_err!("{}: unable to read line: {}", function_name, err)
                }
            }
        }
    })?))
}

fn entry_readlines(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let entry_path = get_path_from_entry(value, "FileEntry:readlines()")?;
    readlines(luau, &entry_path, "FileEntry:readlines()")
}

fn entry_metadata(luau: &Lua, value: LuaValue) -> LuaValueResult {
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

fn entry_is_valid_utf8(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    let entry_path = get_path_from_entry(value, "FileEntry:is_valid_utf8()")?;
    let mut file = match fs::File::open(&entry_path) {
        Ok(file) => file,
        Err(err) => {
            return wrap_io_read_errors(err, "FileEntry:is_valid_utf8()", &entry_path);
        }
    };
    let mut buffer = Vec::new();
    match file.read_to_end(& mut buffer) {
        Ok(_) => {},
        Err(err) => {
            return wrap_err!("FileEntry:is_valid_utf8(): error reading file: {}", err);
        }
    };
    match std::str::from_utf8(&buffer) {
        Ok(_) => Ok(LuaValue::Boolean(true)),
        Err(_) => Ok(LuaValue::Boolean(false)),
    }
}

fn entry_copy_to(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
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

    if metadata.is_dir() {
        match copy_dir(&entry_path, destination_path) {
            Ok(_unsuccessful) => {
                Ok(())
            },
            Err(err) => {
                wrap_io_read_errors_empty(err, "Entry:copy_to()", &entry_path)
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

pub fn create_file_entry(luau: &Lua, path: String) -> LuaResult<LuaTable> {
    let original_path = path.clone();
    let path = PathBuf::from(path);
    if !path.exists() {
        return wrap_err!("File not found: '{}'", path.display());
    }
    let base_name = match path.file_name() {
        Some(name) => {
            match name.to_str() {
                Some(name) => name,
                None => {
                    return wrap_err!("unable to create FileEntry; the name of the file at path {} is non-unicode", path.display());
                }
            }
        },
        None => "",
    };

    TableBuilder::create(luau)?
        .with_value("name", base_name)?
        .with_value("type", "File")?
        .with_value("path", original_path)?
        .with_function("size", entry_filesize)?
        .with_function("read", entry_readfile)?
        .with_function("readbytes", entry_readbytes)?
        .with_function("readlines", entry_readlines)?
        .with_function("is_valid_utf8", entry_is_valid_utf8)?
        .with_function("append", entry_append)?
        .with_function("metadata", entry_metadata)?
        .with_function("copy_to", entry_copy_to)?
        .build_readonly()
}