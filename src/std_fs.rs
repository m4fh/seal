use mlua::prelude::*;
use std::fs::remove_dir_all;
use std::{fs, io};
use io::Write;

use regex::Regex;
use crate::{table_helpers::TableBuilder, LuaValueResult};
use crate::{err_handling as errs, wrap_err, std_io_colors as colors};

fn list_dir(luau: &Lua, path: String) -> LuaResult<LuaTable> {
    match fs::metadata(&path) {
        Ok(t) => {
            if t.is_dir() {
                let entries_list = luau.create_table()?;
                for entry in fs::read_dir(&path)? {
                    let entry = entry?;
                    if let Some(entry_path) = entry.path().to_str() {
                        entries_list.push(entry_path)?;
                    }
                };
                Ok(entries_list)
            } else {
                Err(LuaError::runtime("Attempt to list files/entries on path, but path is a file itself"))
            }
            
        },
        Err(err) => {
            let err_message = match err.kind() {
                io::ErrorKind::NotFound => format!("Invalid directory: \"{}\"", path),
                io::ErrorKind::PermissionDenied => format!("Permission denied: {}", path),
                _ => todo!()
            };
            Err(LuaError::runtime(err_message))
        }
    }
}

fn get_entries(luau: &Lua, directory_path: String) -> LuaResult<LuaTable> {
    match fs::metadata(&directory_path) {
        Ok(t) => {
            if t.is_dir() {
                let entries_dictionary = luau.create_table()?;
                let grab_file_ext_re = Regex::new(r"\.([a-zA-Z0-9]+)$").unwrap();
                for entry in fs::read_dir(&directory_path)? {
                    let entry = entry?;
                    if let Some(entry_path) = entry.path().to_str() {
                        let entry_table = luau.create_table()?;
                        let entry_metadata = entry.metadata().unwrap();
                        if entry_metadata.is_dir() {
                            if entry_metadata.is_dir() {
                                entry_table.set("type", "Directory")?;
                                entry_table.set("path", entry_path)?;
                                entry_table.set("entries", luau.create_function({
                                    let entry_path = entry_path.to_string();
                                    move | luau, _s: LuaMultiValue | {
                                        get_entries(luau, entry_path.clone())
                                    }})?)?;
                                entry_table.set("list", luau.create_function({
                                    let entry_path = entry_path.to_string();
                                    move | luau, _s: LuaMultiValue | {
                                        list_dir(luau, entry_path.clone())
                                    }
                                })?)?;
                            }
                        } else {

                            let extension = {
                                if let Some(captures) = grab_file_ext_re.captures(entry_path) {
                                    String::from(&captures[1])
                                } else {
                                    String::from("")
                                }
                            };

                            entry_table.set("type", "File")?;
                            entry_table.set("path", entry_path)?;
                            entry_table.set("extension", extension)?;
                            entry_table.set("read", luau.create_function({
                                let entry_path = entry_path.to_string();
                                move | _luau, _s: LuaMultiValue | {
                                    Ok(fs::read_to_string(entry_path.clone())?)
                                }
                            })?)?;
                        }
                        entries_dictionary.set(entry_path, entry_table)?;
                        // entries_dictionary.push(entry_path)?;
                    }
                };
                Ok(entries_dictionary)
            } else {
                wrap_err!("Attempt to list files/entries of a path, but path is a file itself!")
                // Err(LuaError::external("Attempt to list files/entries of path, but path is a file itself"))
            }
            
        },
        Err(err) => {
            let err_message = match err.kind() {
                io::ErrorKind::NotFound => format!("Invalid directory: \"{}\"", directory_path),
                io::ErrorKind::PermissionDenied => format!("Permission denied: {}", directory_path),
                _ => todo!()
            };
            Err(LuaError::runtime(err_message))
        }
    }
}

pub fn read_file(_: &Lua, file_path: String) -> LuaResult<String> {
    let result = match fs::read_to_string(&file_path) {
        Ok(content) => Ok(content),
        Err(err) => {
            let err_message = match err.kind() {
                io::ErrorKind::NotFound => format!("File not found: {}", file_path),
                io::ErrorKind::PermissionDenied => format!("Permission denied: {}", file_path),
                _ => todo!()
            };
            Err(LuaError::external(err_message))
        }
    };
    Ok(result?)
}

/**
expects table in format of
```luau
type WriteFileOptions = {
    path: string,
    content: string,
    overwrite: boolean?,
}
```
*/
pub fn write_file(_luau: &Lua, write_file_options: LuaValue) -> LuaValueResult {
    match write_file_options {
        LuaValue::Table(options) => {
            let file_path = match options.get("path")? {
                LuaValue::String(p) => p.to_string_lossy(),
                other => {
                    panic!("WriteFileOptions expected path to be a string, got: {:?}", other);
                }
            };
            let file_content = match options.get("content")? {
                LuaValue::String(c) => c.to_string_lossy(),
                other => {
                    panic!("WriteFileOptions expected content to be a string, got: {:?}", other);
                }
            };
            let should_overwrite = match options.get("overwrite")? {
                LuaValue::Boolean(b) => if b == true {true} else {false},
                LuaValue::Nil => true,
                other => {
                    panic!("WriteFileOptions expected overwrite to be a boolean or nil, got: {:?}", other);
                }
            };

            if !fs::metadata(file_path.clone()).is_ok() || should_overwrite {
                let mut new_file = fs::File::create(file_path)?;
                new_file.write_all(file_content.as_bytes())?;
                Ok(LuaNil)
            } else {
                let err_message = format!("{:?} already exists! Use WriteFileOptions.overwrite = true to overwrite.", file_path);
                Err(LuaError::external(err_message))
            }
        },
        _ => {
            let err_message = format!("fs.writefile expected WriteFileOptions table ({{path: string, content: string, overwrite: boolean?}}, got {:?})", write_file_options);
            Err(LuaError::external(err_message))
        }
    }

}

fn does_file_exist(file_path: String) -> bool {
    fs::metadata(&file_path).is_ok()
}

fn exists(_luau: &Lua, file_path: String) -> LuaResult<bool> {
    if does_file_exist(file_path) {
        Ok(true)
    } else {
        Ok(false)
    }
}

fn create_entry_table(luau: &Lua, entry_path: &str) -> LuaResult<LuaTable> {
    let grab_file_ext_re = Regex::new(r"\.([a-zA-Z0-9]+)$").unwrap();
    let metadata = fs::metadata(entry_path)?;
    let entry_table = luau.create_table()?;
    if metadata.is_dir() {
        entry_table.set("type", "Directory")?;
        entry_table.set("path", entry_path)?;
        entry_table.set("entries", luau.create_function({
          let entry_path = entry_path.to_string();
            move | luau, _s: LuaMultiValue | {
                get_entries(luau, entry_path.clone())
            }
        })?)?;
        entry_table.set("list", luau.create_function({
        let entry_path = entry_path.to_string();
            move | luau, _s: LuaMultiValue | {
                list_dir(luau, entry_path.clone())
            }
        })?)?;
        entry_table.set("remove", luau.create_function({
            let entry_path = entry_path.to_string();
            move | _luau, _s: LuaMultiValue | {
                Ok(fs::remove_dir_all(entry_path.clone())?)
            }
        })?)?;
    } else {
        let extension = {
            if let Some(captures) = grab_file_ext_re.captures(entry_path) {
                String::from(&captures[1])
            } else {
                String::from("")
            }
        };

        entry_table.set("type", "File")?;
        entry_table.set("path", entry_path)?;
        entry_table.set("extension", extension)?;
        entry_table.set("read", luau.create_function({
            let entry_path = entry_path.to_string();
            move | _luau, _s: LuaMultiValue | {
                Ok(fs::read_to_string(entry_path.clone())?)
            }
        })?)?;
        entry_table.set("remove", luau.create_function({
            let entry_path = entry_path.to_string();
            move | _luau, _s: LuaMultiValue | {
                Ok(fs::remove_file(entry_path.clone())?)
                // Ok(fs::read_to_string(entry_path.clone())?)
            }
        })?)?;
    }
    Ok(entry_table)
}

fn is_dir_empty(path: &str) -> bool {
    match fs::read_dir(&path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => {
            panic!("Error reading path {}", &path);
        }, 
    }
}

pub fn fs_remove(_luau: &Lua, remove_options: LuaValue) -> LuaValueResult {
    match remove_options {
        LuaValue::Table(options) => {
            if let LuaValue::String(path) = options.get("file")? {
                let path = path.to_str()?.to_string();
                fs::remove_file(&path)?;
                Ok(LuaNil)
            } else if let LuaValue::String(directory_path) = options.get("directory")? {
                let directory_path = directory_path.to_str()?.to_string();
                // let force = options.get("force")?;

                match options.get("force")? {
                    LuaValue::Boolean(force) => {
                        if force {
                            if directory_path.starts_with("/") {
                                match options.get("remove_absolute_path")? {
                                    LuaValue::Boolean(safety_override) => {
                                        if safety_override {
                                            remove_dir_all(&directory_path)?;
                                        } else {
                                            return wrap_err!("fs.remove: attempted to remove a directory by absolute path.\nThis could be a critical directory, so please be careful. Directory: {}. If you're absolutely sure your code cannot unintentionally destroy a critical directory like /, /root, /boot, or on windows, C:\\System32 or something, then feel free to set RemoveDirectoryOptions.remove_absolute_path to true.", &directory_path);
                                        }
                                    }, 
                                    other => {
                                        return wrap_err!("fs.remove: remove_absolute_path expected to be boolean (default false), you gave me a {:?}, why?", other);
                                    }
                                }
                            } else {
                                fs::remove_dir_all(&directory_path)?;
                            }
                        } else {
                            if is_dir_empty(&directory_path) {
                                fs::remove_dir(&directory_path)?;
                            }
                        }
                    },
                    LuaValue::Nil => {
                        fs::remove_dir_all(&directory_path)?;
                    },
                    other => {
                        return wrap_err!("fs.remove expected RemoveDirectoryOptions.force to be string? (string or the default, nil), got: {:?}", other);
                    }
                }

                Ok(LuaNil)
            } else {
                errs::wrap("fs.remove received invalid arguments; expected RemoveOptions.file or RemoveOptions.directory.")
            }
            
        },
        other => {
            wrap_err!("fs.remove expected RemoveOptions, got: {}", other.to_string()?)
        }
    }
}

fn fs_find(luau: &Lua, query: LuaValue) -> LuaValueResult {
    match query {
        LuaValue::String(q) => {
            let q = q.to_str()?.to_string();
            if exists(luau, q.clone())? {
                Ok(LuaValue::Table(create_entry_table(luau, &q)?))
            } else {
                Ok(LuaNil)
            }
        },
        LuaValue::Table(q) => {
            if let LuaValue::String(dir_path) = q.get("directory")? {
                let dir_path = dir_path.to_str()?.to_string();
                let dir_metadata = fs::metadata(&dir_path);
                if dir_metadata.is_ok() {
                    if dir_metadata?.is_dir() {
                        Ok(LuaValue::Table(create_entry_table(luau, &dir_path)?))
                    } else {
                        wrap_err!("fs.find: {} exists but is not a directory!", &dir_path)
                    }
                } else {
                    Ok(LuaNil)
                }
            } else if let LuaValue::String(file_path) = q.get("file")? {
                let file_path = file_path.to_str()?.to_string();
                let file_metadata = fs::metadata(&file_path);
                if file_metadata.is_ok() {
                    if file_metadata?.is_file() {
                        Ok(LuaValue::Table(create_entry_table(luau, &file_path)?))
                    } else {
                        wrap_err!("fs.find: {} exists but is not a file!", &file_path)
                    }
                } else {
                    Ok(LuaNil)
                }
            } else {
                wrap_err!("fs.find expected to be called with either a string (file or directory path) or a table of type {{file: string}} | {{directory: string}}")
            }
        },
        other => {
            let err_message = format!("fs.find expected string or FindQuery, got: {:?}", other);
            Err(LuaError::external(err_message))
        }
    }
}

fn fs_create(luau: &Lua, new_options: LuaValue) -> LuaValueResult {
    match new_options {
        LuaValue::Table(options) => {
            if let LuaValue::String(file_path) = options.get("file")? {
                let writefile_options = TableBuilder::create(luau)?
                    .with_value("file", file_path)?
                    .with_value("content", "")?
                    .build()?;
                write_file(luau, LuaValue::Table(writefile_options))?;
                Ok(LuaNil)
            } else if let LuaValue::String(directory_path) = options.get("directory")? {
                let directory_path = directory_path.to_str()?.to_string();
                fs::create_dir(directory_path)?;
                Ok(LuaNil)
            } else {
                wrap_err!("fs.create expected table fields 'file' or 'directory', got neither")
            }
        },
        other => {
            wrap_err!("fs.create expected to be called with table of type {{ file: string }} or {{ directory: string }}, got {:?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let std_fs = TableBuilder::create(luau)?
        .with_function("readfile", read_file)?
        .with_function("writefile", write_file)?
        .with_function("remove", fs_remove)?
        .with_function("exists", exists)?
        .with_function("list", list_dir)?
        .with_function("entries", get_entries)?
        .with_function("find", fs_find)?
        .with_function("create", fs_create)?
        .build_readonly()?;

    Ok(std_fs)
}
