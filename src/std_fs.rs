use mlua::prelude::*;
use std::{fs, io};
use io::Write;

use regex::Regex;
use crate::table_helpers::TableBuilder;

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
                Err(LuaError::external("Attempt to list files/entries of path, but path is a file itself"))
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
pub fn write_file(_luau: &Lua, write_file_options: LuaValue) -> LuaResult<LuaValue> {
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
                LuaValue::Nil => false,
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
    Ok(entry_table)
}

fn fs_find(luau: &Lua, query: LuaValue) -> LuaResult<LuaValue> {
    match query {
        LuaValue::String(q) => {
            let q = q.to_str()?.to_string();
            if fs::exists(&q)? {
                Ok(LuaValue::Table(create_entry_table(luau, &q)?))
            } else {
                Ok(LuaNil)
            }
        },
        LuaValue::Table(q) => {
            if let LuaValue::String(dir_path) = q.get("directory")? {
                let dir_path = dir_path.to_str()?.to_string();
                if !fs::exists(&dir_path)? {
                    Ok(LuaNil)
                } else if fs::metadata(&dir_path)?.is_dir() {
                    Ok(LuaValue::Table(create_entry_table(luau, &dir_path)?))
                } else {
                    Err(LuaError::external(format!("fs.find: {} is not a directory", &dir_path)))
                }
            } else if let LuaValue::String(file_path) = q.get("file")? {
                let file_path = file_path.to_str()?.to_string();
                if !fs::exists(&file_path)? {
                    Ok(LuaNil)
                } else if fs::metadata(&file_path)?.is_file() {
                    Ok(LuaValue::Table(create_entry_table(luau, &file_path)?))
                } else {
                    Err(LuaError::external(format!("fs.find: {} is not a file", &file_path)))
                }
            } else {
                Err(LuaError::external("fs.find{} expected to be called with keys 'directory' or 'file', got neither"))
            }
        },
        other => {
            let err_message = format!("fs.find expected string or FindQuery, got: {:?}", other);
            Err(LuaError::external(err_message))
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let std_fs = TableBuilder::create(luau)?
        .with_function("readfile", read_file)?
        .with_function("writefile", write_file)?
        .with_function("exists", exists)?
        .with_function("list", list_dir)?
        .with_function("entries", get_entries)?
        .with_function("find", fs_find)?
        .build_readonly()?;

    Ok(std_fs)
}
