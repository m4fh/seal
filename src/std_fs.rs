use mlua::prelude::*;
use std::io::{BufRead, Read, Seek};
use std::{fs, io};
use io::Write;

use regex::Regex;
use crate::{table_helpers::TableBuilder, LuaValueResult};
use crate::{wrap_err, std_io_colors as colors};

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

fn fs_entries(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let directory_path = match value {
        LuaValue::String(directory_path) => directory_path.to_string_lossy(),
        other => {
            return wrap_err!("fs.entries(directory_path: string) expected directory_path to be string, got: {:#?}", other);
        }
    };

    let metadata = match fs::metadata(&directory_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_err!("fs.entries: error reading directory_path's metadata: {}", err);
        }
    };

    if metadata.is_dir() {
        let mut entry_vec: Vec<(String, LuaValue)> = Vec::new();
        for entry in fs::read_dir(directory_path)? {
            let entry = entry?;
            let entry_path = entry.path()
                .to_str()
                .unwrap()
                .to_string();
            entry_vec.push((entry_path.to_owned(), LuaValue::Table(
                create_entry_table(luau, &entry_path)?
            )));
            
        };
        Ok(LuaValue::Table(
            TableBuilder::create(luau)?
                .with_values(entry_vec)?
                .build_readonly()?
        ))
    } else {
        wrap_err!("fs.entries: expected directory, but path ({}) is actually a file", directory_path)
    }
}

pub fn fs_readfile(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = match value {
        LuaValue::String(file_path) => file_path.to_string_lossy(),
        other => {
            return wrap_err!("fs.readfile expected string, got {:#?}", other);
        }
    };
    let bytes = match fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound =>
                    return wrap_err!("fs.readfile: File not found: {}", file_path),
                io::ErrorKind::PermissionDenied =>
                    return wrap_err!("fs.readfile: Permission denied: {}", file_path),
                other => {
                    return wrap_err!("fs.readfile: Error reading file: {}", other);
                }
            }
        }
    };
    Ok(LuaValue::String(luau.create_string(bytes)?))
}

// Reads random file into an array of bytes
pub fn fs_readbytes(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let entry_path: String = match multivalue.pop_front() {
        Some(LuaValue::String(file_path)) => file_path.to_string_lossy(),
        Some(other) => 
            return wrap_err!("fs.readbytes(file_path, s: number?, f: number?) expected file path to be a string, got: {:#?}", other),
        None => {
            return wrap_err!("fs.readbytes(file_path, s: number?, f: number?) expected to be called with self.");
        }
    };

    let start = match multivalue.pop_front() {
        Some(LuaValue::Integer(n)) => {
            if n >= 0 {
                Some(n)
            } else {
                return wrap_err!("fs.readbytes: start byte s must be >= 0!!");
            }
        },
        Some(other) => return wrap_err!("fs.readbytes(file_path, s: number?, f: number?) expected s to be a number, got: {:#?}", other),
        None => None,
    };
    let finish = match multivalue.pop_front() {
        Some(LuaValue::Integer(n)) => {
            if n > 0 { 
                Some(n)
            } else {
                return wrap_err!("fs.readbytes: final byte f must be positive!!");
            }
        },
        Some(other) => return wrap_err!("fs.readbytes(file_path, s: number?, f: number?) expected f to be a number, got: {:#?}", other),
        None => {
            if start.is_some() {
                return wrap_err!("fs.readbytes(file_path, s: number, f: number): missing final byte f; if s is provided then f must also be provided");
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
                return wrap_err!("fs.readbytes(file_path, s: number?, f: number?) error reading path: {}", err);
            }
        };
    
        // Calculate the number of bytes to read
        let num_bytes = (finish - start) as usize;
        let mut buffer = vec![0; num_bytes];
    
        // Seek to the start position
        if let Err(err) = file.seek(std::io::SeekFrom::Start(start as u64)) {
            return wrap_err!("fs.readbytes: error seeking to start position: {}", err);
        }
    
        // Read the requested bytes
        match file.read_exact(&mut buffer) {
            Ok(_) => {
                // let lua_string = luau.create_string(&buffer)?;
                let buffy = luau.create_buffer(&buffer)?;
                Ok(LuaValue::Buffer(buffy))
            },
            Err(err) => wrap_err!("fs.readbytes: error reading bytes: {}", err),
        }
    } else {
        // read the whole thing
        let bytes = match fs::read(&entry_path) {
            Ok(bytes) => bytes,
            Err(err) => {
                return wrap_err!("fs.readbytes: failed to read file with error: {}", err);
            }
        };
        let buffy = luau.create_buffer(bytes)?;
        Ok(LuaValue::Buffer(buffy))
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
                LuaValue::Boolean(b) => b,
                LuaValue::Nil => true,
                other => {
                    panic!("WriteFileOptions expected overwrite to be a boolean or nil, got: {:?}", other);
                }
            };

            if fs::metadata(file_path.clone()).is_err() || should_overwrite {
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
    fs::metadata(file_path).is_ok()
}

fn create_entry_table(luau: &Lua, entry_path: &str) -> LuaResult<LuaTable> {
    let grab_file_ext_re = Regex::new(r"\.([a-zA-Z0-9]+)$").unwrap();
    let metadata = fs::metadata(entry_path)?;
    if metadata.is_dir() {
        TableBuilder::create(luau)?
            .with_value("type", "Directory")?
            .with_value("path", entry_path)?
            .with_function("entries", {
                let entry_path = entry_path.to_string();
                let entry_path_but_in_luau = entry_path.into_lua(luau)?;
                move | luau, _s: LuaMultiValue | {
                    fs_entries(luau, entry_path_but_in_luau.to_owned())
                }
            })?
            .with_function("list", {
                let entry_path = entry_path.to_string();
                move | luau, _s: LuaMultiValue | {
                    fs_listdir(luau, entry_path.clone())
                }
            })?
            .with_function("find", {
                let entry_path = entry_path.to_string();
                move | luau, mut multivalue: LuaMultiValue | {
                    let _self = multivalue.pop_front();
                    let find_options = multivalue.pop_front().unwrap();
                    match find_options {
                        LuaValue::String(find_path) => {
                            let new_path = format!("{entry_path}/{}", find_path.to_str()?);
                            Ok(fs_find(luau, new_path.into_lua(luau)?))
                        }, 
                        LuaValue::Table(find_table) => {
                            if let LuaValue::String(file_path) = find_table.get("file")? {
                                let new_path = format!("{entry_path}/{}", file_path.to_str()?);
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
            })?
            .with_function("remove", {
                let entry_path = entry_path.to_string();
                move | _luau, _s: LuaMultiValue | {
                    Ok(fs::remove_dir_all(entry_path.clone())?)
                }
            })?
            .with_function("create", {
                let entry_path = entry_path.to_string();
                move | luau, mut multivalue: LuaMultiValue | {
                    let _entry = multivalue.pop_front();
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
                                // println!("{:#?}", v);
                                todo!("tree creation not yet implemented")
                            }
                        },
                        other => wrap_err!("DirectoryEntry:create for {} expected to be called with a table containing key 'dictionary' or key 'string', got {:?}", &entry_path, other)
                    };
                    fs_create(luau, prepended_entry?)
                }
            })?
            .build_readonly()
    } else {
        let extension = {
            if let Some(captures) = grab_file_ext_re.captures(entry_path) {
                String::from(&captures[1])
            } else {
                String::from("")
            }
        };

        TableBuilder::create(luau)?
            .with_value("type", "File")?
            .with_value("size", metadata.len())?
            .with_value("path", entry_path)?
            .with_value("extension", extension)?
            .with_function("read", {
                let entry_path = entry_path.to_string();
                move | _luau, _s: LuaMultiValue | {
                    Ok(fs::read_to_string(entry_path.clone())?)
                }
            })?
            .with_function("readbytes", {
                let entry_path = entry_path.to_string();
                move | luau: &Lua, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    let _handle = match multivalue.pop_front() {
                        Some(value) => value,
                        None => {
                            return wrap_err!("FileEntry:readbytes(s, f) expected to be called with self.");
                        }
                    };
                    let entry_path_luau = luau.create_string(&entry_path)?;
                    multivalue.push_front(LuaValue::String(entry_path_luau));
                    match fs_readbytes(luau, multivalue) {
                        Ok(v) => Ok(v),
                        Err(err) => {
                            wrap_err!(err.to_string().replace("fs.readbytes(file_path,", "FileEntry:readbytes("))
                        }
                    }
                }   
            })?
            .with_function("readlines", {
                let entry_path = entry_path.to_string();
                move | luau, mut multivalue: LuaMultiValue | -> LuaValueResult {
                    let file = match fs::File::open(&entry_path) {
                        Ok(file) => file,
                        Err(err) =>{
                            return wrap_err!("FileEntry:readlines: error opening file: {}", err);
                        }
                    };
                    
                    let reader = io::BufReader::new(file);
                    match multivalue.pop_back() {
                        Some(LuaValue::Function(handler_function)) => {
                            for (index, line) in reader.lines().enumerate() {
                                match line {
                                    Ok(line) => {
                                        let line = line.into_lua(luau)?;
                                        let index = LuaValue::Integer((index + 1) as i32);
                                        let args = LuaMultiValue::from_vec(vec![line, index]);
                                        match handler_function.call::<Option<LuaString>>(args) {
                                            Ok(Some(s)) => {
                                                match s.to_string_lossy().as_str() {
                                                    "break" => break,
                                                    _other => continue,
                                                }
                                            },
                                            Ok(None) => continue,
                                            Err(err) => {
                                                return wrap_err!("error calling readlines callback: {}", err);
                                            }
                                        }
                                    },
                                    Err(err) => {
                                        return wrap_err!("error reading lines: {}", err);
                                    }
                                }
                            }
                            Ok(LuaNil)
                        },
                        Some(other) => {
                            wrap_err!("expected function, got: {:#?}", other)
                        },
                        None => {
                            wrap_err!("expected function, got: nothing")
                        }
                    }
                }
            })?
            .with_function("remove", {
                let entry_path = entry_path.to_string();
                move | _luau, _s: LuaMultiValue | {
                    Ok(fs::remove_file(entry_path.clone())?)
                }
            })?
            .build_readonly()
    }
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
    match fs::read_dir(path) {
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
                        } else if is_dir_empty(&directory_path) {
                            fs::remove_dir(&directory_path)?;
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
                wrap_err!("fs.remove received invalid arguments; expected RemoveOptions.file or RemoveOptions.directory.")
            }
        },
        other => {
            wrap_err!("fs.remove expected RemoveOptions, got: {}", other.to_string()?)
        }
    }
}


// TODO: refactor/fix this. 
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

fn fs_find_file(luau: &Lua, path: LuaValue) -> LuaValueResult {
    let path = match path {
        LuaValue::String(path) => {
            path.to_string_lossy()
        },
        other => {
            return wrap_err!("fs.file expected string (path of the file to look for), got: {:#?}", other);
        }
    };

    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return Ok(LuaNil);
                },
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("fs.file: attempted to find file at path '{}' but permission denied", path);
                },
                other => {
                    return wrap_err!("fs.file: error getting metadata for file at path '{}': {:?}", path, other);
                }
            }
        }
    };

    if metadata.is_file() {
        Ok(LuaValue::Table(
            create_entry_table(luau, &path)?
        ))
    } else if metadata.is_dir() {
        wrap_err!("fs.file: requested file at path '{}' is actually a directory", path)
    } else {
        unreachable!()
    }
}

fn fs_find_dir(luau: &Lua, path: LuaValue) -> LuaValueResult {
    let path = match path {
        LuaValue::String(path) => {
            path.to_string_lossy()
        },
        other => {
            return wrap_err!("fs.dir expected string (path of the directory to look for), got: {:#?}", other);
        }
    };

    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(err) => {
            match err.kind() {
                io::ErrorKind::NotFound => {
                    return Ok(LuaNil);
                },
                io::ErrorKind::PermissionDenied => {
                    return wrap_err!("fs.dir: attempted to find directory at path '{}' but permission denied", path);
                },
                other => {
                    return wrap_err!("fs.dir: error getting metadata for directory at path '{}': {:?}", path, other);
                }
            }
        }
    };

    if metadata.is_dir() {
        Ok(LuaValue::Table(
            create_entry_table(luau, &path)?
        ))
    } else if metadata.is_file() {
        wrap_err!("fs.dir: requested directory at path '{}' is actually a file", path)
    } else {
        unreachable!()
    }
}

fn fs_exists(_luau: &Lua, path: LuaValue) -> LuaValueResult {
    let path = match path {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("fs.exists(path) expected path to be a string, got: {:#?}", other);
        }
    };

    match fs::exists(&path) {
        Ok(true) => Ok(LuaValue::Boolean(true)),
        Ok(false) => Ok(LuaValue::Boolean(false)),
        Err(err) => {
            match err.kind() {
                io::ErrorKind::PermissionDenied => {
                    wrap_err!("fs.exists: attempt to check if path '{}' exists but permission denied", path)
                },
                other => {
                    wrap_err!("fs.exists: encountered an error checking if '{}' exists: {}", path, other)
                }
            }
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
        .with_function("file", fs_find_file)?
        .with_function("dir", fs_find_dir)?
        .with_function("create", fs_create)?
        .with_function("exists", fs_exists)?
        .with_function("readbytes", fs_readbytes)?
        .build_readonly()?;

    Ok(std_fs)
}
