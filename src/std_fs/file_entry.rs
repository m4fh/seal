use mlua::prelude::*;
use crate::{LuaValueResult, LuaEmptyResult, wrap_err};
use crate::{colors, table_helpers::TableBuilder};
use crate::std_fs::entry::{self, wrap_io_read_errors, wrap_io_read_errors_empty, get_path_from_entry};
use std::cell::RefCell;
use std::fs::{self, OpenOptions};
use std::path::PathBuf;
use std::io::{BufRead, BufReader, Read, Seek, Write};

fn file_readfile(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = get_path_from_entry(value, "FileEntry:read()")?;
    let bytes = match fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return wrap_io_read_errors(err,"FileEntry:read()", &file_path);
        }
    };
    Ok(LuaValue::String(luau.create_string(bytes)?))
}

// helper function for fs.readbytes and Entry:readbytes
pub fn read_entry_path_into_buffer(luau: &Lua, entry_path: String, mut multivalue: LuaMultiValue, function_name: &str) -> LuaValueResult {
    let file_size = {
        match fs::metadata(&entry_path) {
            Ok(metadata) => metadata.len() as i32,
            Err(err) => {
                return entry::wrap_io_read_errors(err, function_name, &entry_path);
            }
        }
    };
    let start = match multivalue.pop_front() {
        Some(LuaValue::Integer(n)) => {
            if n >= 0 {
                Some(n)
            } else if n > file_size {
                return wrap_err!("{}: start byte s ({}) outside file bounds ({})", function_name, n, file_size);
            } else {
                return wrap_err!("{}: start byte s must be >= 0!!", function_name);
            }
        },
        Some(other) => return wrap_err!("{}(file_path, s: number?, f: number?) expected s to be a number, got: {:#?}", function_name, other),
        None => None,
    };
    let finish = match multivalue.pop_front() {
        Some(LuaValue::Integer(n)) => {
            if n > 0 { 
                Some(n)
            } else if n > file_size {
                return wrap_err!("{}: final byte f ({}) outside file bounds ({})", function_name, n, file_size);
            } else {
                return wrap_err!("{}: final byte f must be positive!!", function_name);
            }
        },
        Some(other) => return wrap_err!("{}(file_path, s: number?, f: number?) expected f to be a number, got: {:#?}", function_name, other),
        None => {
            if start.is_some() {
                return wrap_err!("{}(file_path, s: number, f: number): missing final byte f; if s is provided then f must also be provided", function_name);
            } else {
                None
            }
        },
    };

    if let Some(start) = start {
        // read specific section of file
        let finish = finish.unwrap();

        let mut file = match fs::File::open(&entry_path) {
            Ok(f) => f,
            Err(err) => {
                return wrap_err!("{}(file_path, s: number?, f: number?) error reading path: {}", function_name, err);
            }
        };
    
        // Calculate the number of bytes to read
        let num_bytes = (finish - start) as usize;
        let mut buffer = vec![0; num_bytes];
    
        // Seek to the start position
        if let Err(err) = file.seek(std::io::SeekFrom::Start(start as u64)) {
            return wrap_err!("{}: error seeking to start position: {}", function_name, err);
        }
    
        // Read the requested bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                let buffy = luau.create_buffer(&buffer)?;
                Ok(LuaValue::Buffer(buffy))
            },
            Err(err) => wrap_err!("{}: error reading bytes: {}", function_name, err),
        }
    } else {
        // read the whole thing
        let bytes = match fs::read(&entry_path) {
            Ok(bytes) => bytes,
            Err(err) => {
                return wrap_err!("{}: failed to read file with error: {}", function_name, err);
            }
        };
        let buffy = luau.create_buffer(bytes)?;
        Ok(LuaValue::Buffer(buffy))
    }
}

fn file_readbytes(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let entry = match multivalue.pop_front() {
        Some(value) => value,
        None => {
            return wrap_err!("FileEntry:readbytes() incorrectly called with zero arguments");
        }
    };
    let entry_path = get_path_from_entry(entry, "FileEntry:readbytes()")?;

    read_entry_path_into_buffer(luau, entry_path, multivalue, "FileEntry:readbytes")
}

fn file_append(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
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
            return entry::wrap_io_read_errors(err, function_name, entry_path);
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

fn file_readlines(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let entry_path = get_path_from_entry(value, "FileEntry:readlines()")?;
    readlines(luau, &entry_path, "FileEntry:readlines()")
}

fn file_filesize(_luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = get_path_from_entry(value, "FileEntry:size()")?;
    let metadata = match fs::metadata(&file_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors(err, "FileEntry:size()", &file_path);
        }
    };
    Ok(LuaValue::Number(metadata.len() as f64))
}

fn file_is_valid_utf8(_luau: &Lua, value: LuaValue) -> LuaValueResult {
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

pub fn create(luau: &Lua, path: String) -> LuaResult<LuaTable> {
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
        .with_function("size", file_filesize)?
        .with_function("read", file_readfile)?
        .with_function("readbytes", file_readbytes)?
        .with_function("readlines", file_readlines)?
        .with_function("is_valid_utf8", file_is_valid_utf8)?
        .with_function("append", file_append)?
        .with_function("metadata", entry::metadata)?
        .with_function("copy_to", entry::copy_to)?
		.with_function("move_to", entry::move_to)?
		.with_function("rename", entry::rename)?
		.with_function("remove", entry::remove)?
        .build_readonly()
}