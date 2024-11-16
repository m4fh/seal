use mlua::prelude::*;
use std::{fs, io};
use io::Write;

use regex::Regex;
use crate::{table_helpers::TableBuilder, LuaValueResult};
use crate::{err_handling as errs, wrap_err, std_io_colors as colors};

fn fs_listdir(luau: &Lua, path: String) -> LuaResult<LuaTable> {
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

fn fs_entries(luau: &Lua, directory_path: String) -> LuaResult<LuaTable> {
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
                            entry_table.set("type", "Directory")?;
                            entry_table.set("path", entry_path)?;
                            entry_table.set("entries", luau.create_function({
                                let entry_path = entry_path.to_string();
                                move | luau, _s: LuaMultiValue | {
                                    fs_entries(luau, entry_path.clone())
                                }})?)?;
                            entry_table.set("list", luau.create_function({
                                let entry_path = entry_path.to_string();
                                move | luau, _s: LuaMultiValue | {
                                    fs_listdir(luau, entry_path.clone())
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
                        entries_dictionary.set(entry_path, entry_table)?;
                        // entries_dictionary.push(entry_path)?;
                    }
                };
                Ok(entries_dictionary)
            } else {
                wrap_err!("fs.entries: Attempt to list files/entries of a path, but path is a file itself!")
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

pub fn fs_readfile(_: &Lua, file_path: String) -> LuaResult<String> {
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
pub fn fs_writefile(_luau: &Lua, write_file_options: LuaValue) -> LuaValueResult {
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
            wrap_err!("fs.writefile expected WriteFileOptions table ({{path: string, content: string, overwrite: boolean?}}, got {:?})", write_file_options)
        }
    }

}

fn does_file_exist(file_path: &str) -> bool {
    fs::metadata(&file_path).is_ok()
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
                fs_entries(luau, entry_path.clone())
            }
        })?)?;
        entry_table.set("list", luau.create_function({
            let entry_path = entry_path.to_string();
            move | luau, _s: LuaMultiValue | {
                fs_listdir(luau, entry_path.clone())
            }
        })?)?;
        entry_table.set("find", luau.create_function({
            let entry_path = entry_path.to_string();
            move | luau, mut multivalue: LuaMultiValue | {
                let _self = multivalue.pop_front();
                let find_options = multivalue.pop_front().unwrap();
                match find_options {
                    LuaValue::String(find_path) => {
                        let new_path = format!("{entry_path}/{}", find_path.to_str()?.to_string());
                        Ok(fs_find(luau, new_path.into_lua(luau)?))
                    }, 
                    LuaValue::Table(find_table) => {
                        if let LuaValue::String(file_path) = find_table.get("file")? {
                            let new_path = format!("{entry_path}/{}", file_path.to_str()?.to_string());
                            find_table.set("file", new_path)?;
                        }
                        Ok(fs_find(luau, LuaValue::Table(find_table)))
                    },
                    other => {
                        wrap_err!("DirectoryEntry:find expected string or FindConfig table, got: {:?}", other)
                    }
                }
                // fs_listdir(luau, entry_path.clone())
            }
        })?)?;
        entry_table.set("remove", luau.create_function({
            let entry_path = entry_path.to_string();
            move | _luau, _s: LuaMultiValue | {
                Ok(fs::remove_dir_all(entry_path.clone())?)
            }
        })?)?;
        entry_table.set("create", luau.create_function({
            let entry_path = entry_path.to_string();
            move | luau, mut multivalue: LuaMultiValue | {
                let _self = multivalue.pop_front();
                let value = multivalue.pop_front().unwrap();
                let prepended_entry = match value {
                    LuaValue::Table(v) => {
                        if let LuaValue::String(new_path) = v.get("directory")? {
                            let new_path = new_path.to_str()?.to_string();
                            v.set("directory", format!("{entry_path}/{new_path}"))?;
                            Ok(LuaValue::Table(v))
                        } else if let LuaValue::String(new_path) = v.get("file")? {
                            let new_path = new_path.to_str()?.to_string();
                            v.set("file",format!("{entry_path}/{new_path}"))?;
                            Ok(LuaValue::Table(v))
                        } else if let LuaValue::Table(file_info) = v.get("file")? {
                            if let LuaValue::String(file_name) = file_info.get("name")? {
                                let file_name = file_name.to_str()?.to_string();
                                let new_path = format!("{entry_path}/{file_name}");
                                file_info.set("name", new_path)?;
                            };
                            Ok(LuaValue::Table(v))
                        } else {
                            println!("{:#?}", v);
                            todo!()
                        }
                    },
                    other => wrap_err!("DirectoryEntry:create for {} expected to be called with a table containing key 'dictionary' or key 'string', got {:?}", &entry_path, other)
                };
                fs_create(luau, prepended_entry?)
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
            }
        })?)?;
    }
    Ok(entry_table)
}

pub fn fs_move(_luau: &Lua, from_to: LuaMultiValue) -> LuaValueResult {
    let mut multivalue = from_to.clone();
    let from = multivalue.pop_front().unwrap_or(LuaNil);
    let from_path = {
        match from {
            LuaValue::String(from) => from.to_str()?.to_string(),
            other => {
                return wrap_err!("fs.move: 'from' argument expected to be string path, got {:?}", other);
            }
        }
    };
    let to = multivalue.pop_front().unwrap_or(LuaNil);
    let to_path = {
        match to {
            LuaValue::String(to) => to.to_str()?.to_string(),
            other => {
                return wrap_err!("fs.move: 'to' argument expected to be string path, got {:?}", other)
            }
        }
    };
    std::fs::rename(from_path, to_path)?;
    Ok(LuaNil)
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
                match options.get("force")? {
                    LuaValue::Boolean(force) => {
                        if force {
                            if directory_path.starts_with("/") {
                                match options.get("remove_absolute_path")? {
                                    LuaValue::Boolean(safety_override) => {
                                        if safety_override {
                                            fs::remove_dir_all(&directory_path)?;
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
            if does_file_exist(&q) {
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
            wrap_err!("fs.find expected string or FindQuery, got: {:?}", other)
        }
    }
}

fn fs_create(luau: &Lua, new_options: LuaValue) -> LuaValueResult {
    match new_options {
        LuaValue::Table(options) => {
            let entry_path = {
                if let LuaValue::String(file_path) = options.get("file")? {
                    let writefile_options = TableBuilder::create(luau)?
                        .with_value("path", file_path.to_owned())?
                        .with_value("content", "")?
                        .build_readonly()?;
                    fs_writefile(luau, LuaValue::Table(writefile_options))?;
                    file_path.to_str()?.to_string()
                    // Ok(LuaNil)
                } else if let LuaValue::Table(file_options) = options.get("file")? {
                   let file_name: LuaString = file_options.get("name")?;
                   let file_content: LuaString = file_options.get("content")?;
                   let writefile_options = TableBuilder::create(luau)?
                        .with_value("path", file_name.to_owned())?
                        .with_value("content", file_content)?
                        .build_readonly()?;
                    fs_writefile(luau, LuaValue::Table(writefile_options))?;
                    file_name.to_str()?.to_string()
                    // Ok(LuaNil)
                } else if let LuaValue::String(directory_path) = options.get("directory")? {
                    let dir_path = directory_path.to_string_lossy().to_string();
                    fs::create_dir(&dir_path)?;
                    dir_path
                    // Ok(LuaNil)
                } else if let LuaValue::Table(_tree) = options.get("directory")? {
                    todo!()
                } else {
                    return wrap_err!("fs.create expected {{file: string}} or {{file: {{name: string, content: string}}}}, but got something else");
                }
            };
            Ok(LuaValue::Table(create_entry_table(luau, &entry_path)?))
        },
        other => {
            wrap_err!("fs.create expected to be called with table of type {{ file: string }} or {{ directory: string }}, got {:?}", other)
        }
    }
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let std_fs = TableBuilder::create(luau)?
        .with_function("readfile", fs_readfile)?
        .with_function("writefile", fs_writefile)?
        .with_function("move", fs_move)?
        .with_function("remove", fs_remove)?
        .with_function("list", fs_listdir)?
        .with_function("entries", fs_entries)?
        .with_function("find", fs_find)?
        .with_function("create", fs_create)?
        .build_readonly()?;

    Ok(std_fs)
}
