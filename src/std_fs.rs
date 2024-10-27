use mlua::prelude::*;
use std::{fs, io};
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

fn read_file(_: &Lua, file_path: String) -> LuaResult<String> {
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
fn write_file(_luau: &Lua, t: LuaTable) -> LuaResult<LuaValue> {
    let file_path: LuaValue = t.get("path")?;
    let content: LuaValue = t.get("content")?;
    let _should_overwrite: LuaValue = t.get("overwrite")?;

    if file_path.is_string() && content.is_string() {
        // let result_table = luau.create_table()?;
        // result_table
        Ok(LuaNil)
    } else {
        Err(LuaError::runtime("fs.writefile table must contain keys \"path\" and \"content\""))
    }

}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let std_fs = TableBuilder::create(luau)?
        .with_function("readfile", read_file)?
        .with_function("writefile", write_file)?
        .with_function("exists", exists)?
        .with_function("list", list_dir)?
        .with_function("entries", get_entries)?
        .build_readonly()?;

    Ok(std_fs)
}
