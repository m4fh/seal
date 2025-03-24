use entry::{wrap_io_read_errors, wrap_io_read_errors_empty};
use mlua::prelude::*;
use std::{fs, io};
use crate::require::ok_table;
use crate::{table_helpers::TableBuilder, LuaValueResult};
use crate::{std_io_colors as colors, wrap_err, LuaEmptyResult};

pub mod entry;
pub mod pathlib;
pub mod file_entry;
pub mod directory_entry;

pub fn ensure_utf8_path(path: &LuaString, function_name: &str) -> LuaResult<String> {
    let Ok(path) = path.to_str() else {
        return wrap_err!("{}: provided path '{}' is not properly utf8-encoded", function_name, path.display());
    };
    Ok(path.to_string())
}

/// fs.listdir(path: string, recursive: boolean?): { string }
fn fs_listdir(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaValueResult {
    let dir_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            ensure_utf8_path(&path, "fs.listdir(path: string, recursive: boolean?)")?
        },
        Some(other) => {
            return wrap_err!("fs.listdir(path: string, recursive: boolean?) expected path to be a string, got: {:#?}", other);
        },
        None => {
            return wrap_err!("fs.listdir(path: string, recursive: boolean?) called without any arguments");
        }
    };
    directory_entry::listdir(luau, dir_path, multivalue, "fs.listdir(path: string, recursive: boolean?)")
}

fn fs_entries(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let function_name = "fs.entries(directory: string)";
    let directory_path = match value {
        LuaValue::String(path) => {
            ensure_utf8_path(&path, function_name)?
        },
        other => {
            return wrap_err!("{} expected directory to be a string, got: {:?}", function_name, other);
        }
    };
    let metadata = match fs::metadata(&directory_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors(err, function_name, &directory_path);
        }
    };
    if !metadata.is_dir() {
        return wrap_err!("{} expected '{}' to be a directory, got file instead", function_name, directory_path);
    }

    let mut entry_vec: Vec<(String, LuaValue)> = Vec::new();

    for current_entry in fs::read_dir(&directory_path)? {
        let current_entry = current_entry?;
        let entry_path = current_entry.path().to_string_lossy().to_string();
        // entry::create creates either a FileEntry or DirectoryEntry as needed
        let entry_table = entry::create(luau, &entry_path, function_name)?;
        entry_vec.push((entry_path, entry_table));
    }

    ok_table(TableBuilder::create(luau)?
        .with_values(entry_vec)?
        .build_readonly()
    )
}

/// `fs.readfile(path: string): string`
/// 
/// note that we allow reading invalid utf8 files instead of failing (requiring fs.readbytes) 
/// or replacing with utf8 replacement character
/// 
/// this is because Luau allows strings to be of arbitrary encoding unlike Rust, where they have to be utf8 
pub fn fs_readfile(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = match value {
        LuaValue::String(file_path) => {
            ensure_utf8_path(&file_path, "fs.readfile(path: string)")?
        },
        other => {
            return wrap_err!("fs.readfile(path: string) expected string, got {:#?}", other);
        }
    };
    let bytes = match fs::read(&file_path) {
        Ok(bytes) => bytes,
        Err(err) => {
            return wrap_io_read_errors(err, "fs.readfile(path: string)", &file_path);
        }
    };
    Ok(LuaValue::String(luau.create_string(bytes)?))
}

/// fs.readbytes(path: string, target_buffer: buffer, buffer_offset: number?, file_offset: number?, count: number)
pub fn fs_readbytes(luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let function_name_and_args = "fs.readbytes(path: string, target_buffer: buffer, buffer_offset: number?, file_offset: number?, count: number)";
    let entry_path: String = match multivalue.pop_front() {
        Some(LuaValue::String(file_path)) => {
            ensure_utf8_path(&file_path, function_name_and_args)?
        },
        Some(other) => 
            return wrap_err!("{} expected path to be a string, got: {:#?}", function_name_and_args, other),
        None => {
            return wrap_err!("{} incorrectly called with zero arguments", function_name_and_args);
        }
    };
    file_entry::read_file_into_buffer(luau, &entry_path, multivalue, function_name_and_args)?;
    Ok(())
}

/// iterate over the lines of a file. you can use this within a for loop
/// or put the function this returns in a local and call it repeatedly ala `local nextline = fs.readlines(filepath); local i, line = nextline()`
fn fs_readlines(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let file_path = match value {
        LuaValue::String(path) => {
            ensure_utf8_path(&path, "fs.readlines(path: string)")?
        },
        other => {
            return wrap_err!("fs.readlines(path: string): expected a file path, got: {:#?}", other);
        }
    };
    file_entry::readlines(luau, &file_path, "fs.readlines(path: string)")
}

// fs.writefile(path: string, content: string | buffer): ()
pub fn fs_writefile(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let file_path = match multivalue.pop_front() {
        Some(LuaValue::String(path)) => {
            ensure_utf8_path(&path, "fs.writefile(path: string, content: string | buffer)")?
        },
        Some(other) => {
            return wrap_err!("fs.writefile(path: string, content: string | buffer) expected path to be a string, got: {:#?}", other);
        }
        None => {
            return wrap_err!("fs.writefile(path: string, content: string | buffer) expected path to a be a string, got nothing");
        }
    };
    let content = match multivalue.pop_front() {
        Some(LuaValue::String(content)) => {
            content.as_bytes().to_vec()
        },
        Some(LuaValue::Buffer(content)) => {
            content.to_vec()
        },
        Some(other) => {
            return wrap_err!("fs.writefile(path: string, content: string | buffer) expected content to be a string or buffer, got: {:#?}", other);
        },
        None => {
            return wrap_err!("fs.writefile(path: string, content: string | buffer) expected second argument content to be a string or buffer, got nothing");
        }
    };
    match fs::write(&file_path, &content) {
        Ok(_) => {
            Ok(())
        },
        Err(err) => {
            entry::wrap_io_read_errors_empty(err, "fs.writefile", &file_path)
        }
    }
}

/// fs.removefile(path: string): ()
/// cannot remove directories
pub fn fs_removefile(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let victim_path = match value {
        LuaValue::String(path) => {
            ensure_utf8_path(&path, "fs.removefile(path: string)")?
        },
        other => {
            return wrap_err!("fs.removefile(path: string) expected path to be a string, got: {:?}", other);
        }
    };
    let metadata = match fs::metadata(&victim_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors_empty(err, "fs.removefile(path: string)", &victim_path);
        }
    };
    if metadata.is_file() {
        match fs::remove_file(&victim_path) {
            Ok(_) => Ok(()),
            Err(err) => {
                wrap_io_read_errors_empty(err, "fs.removefile(path: string)", &victim_path)
            }
        }
    } else { // it can't be a symlink as fs::metadata follows symlinks
        wrap_err!("fs.removefile(path: string): cannot remove file; path at '{}' is actually a directory!", victim_path)
    }
}

pub fn fs_move(_luau: &Lua, mut multivalue: LuaMultiValue) -> LuaEmptyResult {
    let from_path = match multivalue.pop_front() {
        Some(LuaValue::String(from)) => {
            ensure_utf8_path(&from, "fs.move(from: string, to: string)")?
        },
        Some(other) => {
            return wrap_err!("fs.move(from: string, to: string) expected 'from' to be a string, got: {:?}", other);
        },
        None => {
            return wrap_err!("fs.move(from: string, to: string) expected 'from', got nothing");
        }
    };
    let to_path = match multivalue.pop_front() {
        Some(LuaValue::String(to)) => {
            ensure_utf8_path(&to, "fs.move(from: string, to: string)")?
        },
        Some(other) => {
            return wrap_err!("fs.move(from: string, to: string) expected 'to' to be a string, got: {:?}", other);
        },
        None => {
            return wrap_err!("fs.move(from: string, to: string) expected 'to', got nothing");
        }
    };
    match fs::rename(&from_path, &to_path) {
        Ok(_) => Ok(()),
        Err(err) => {
            wrap_err!("fs.move: unable to move '{}' -> '{}' due to err: {}", from_path, to_path, err)
        }
    }
}

/// fs.readtree(path: string): DirectoryTree
/// not called readdir because it's uglier + we want dir/tree stuff to autocomplete after file
/// so we want fs.readfile to autocomplete first and i'm assuming it's alphabetical
fn _fs_readtree(_luau: &Lua, _value: LuaValue) -> LuaValueResult {
    todo!()
}

/// fs.writetree(path: string, tree: DirectoryTree): ()
fn _fs_writetree(_luau: &Lua, _value: LuaValue) -> LuaEmptyResult {
    todo!()
}

/// fs.removetree(path: string)
/// does NOT follow symlinks
pub fn fs_removetree(_luau: &Lua, value: LuaValue) -> LuaEmptyResult {
    let function_name = "fs.removetree(path: string)";
    let victim_path = match value {
        LuaValue::String(path) => {
            ensure_utf8_path(&path, function_name)?
        },
        other => {
            return wrap_err!("fs.removetree(path: string) expected path to be a string, got: {:?}", other);
        }
    };
    let metadata = match fs::metadata(&victim_path) {
        Ok(metadata) => metadata,
        Err(err) => {
            return wrap_io_read_errors_empty(err, function_name, &victim_path);
        }
    };
    if metadata.is_dir() {
        if let Err(err) = fs::remove_dir_all(&victim_path) {
            let err_message = "fs.removetree was unable to remove some, or all of the directory tree requested:\n";
            wrap_err!("{}    {}", err_message, err)
        } else {
            Ok(())
        }
    } else {
        wrap_err!("fs.removetree(path: string) expected to find a directory at path '{}' but instead found a file", victim_path)
    }
}


// TODO: refactor/fix this. 
// fn fs_find(luau: &Lua, query: LuaValue) -> LuaValueResult {
//     match query {
//         LuaValue::String(q) => {
//             let q = q.to_str()?.to_string();
//             if does_file_exist(&q) {
//                 Ok(LuaValue::Table(create_entry_table(luau, &q)?))
//             } else {
//                 Ok(LuaNil)
//             }
//         },
//         LuaValue::Table(q) => {
//             if let LuaValue::String(dir_path) = q.get("directory")? {
//                 let dir_path = dir_path.to_str()?.to_string();
//                 let dir_metadata = fs::metadata(&dir_path);
//                 if dir_metadata.is_ok() {
//                     if dir_metadata?.is_dir() {
//                         Ok(LuaValue::Table(create_entry_table(luau, &dir_path)?))
//                     } else {
//                         wrap_err!("fs.find: {} exists but is not a directory!", &dir_path)
//                     }
//                 } else {
//                     Ok(LuaNil)
//                 }
//             } else if let LuaValue::String(file_path) = q.get("file")? {
//                 let file_path = file_path.to_str()?.to_string();
//                 let file_metadata = fs::metadata(&file_path);
//                 if file_metadata.is_ok() {
//                     if file_metadata?.is_file() {
//                         Ok(LuaValue::Table(create_entry_table(luau, &file_path)?))
//                     } else {
//                         wrap_err!("fs.find: {} exists but is not a file!", &file_path)
//                     }
//                 } else {
//                     Ok(LuaNil)
//                 }
//             } else {
//                 wrap_err!("fs.find expected to be called with either a string (file or directory path) or a table of type {{file: string}} | {{directory: string}}")
//             }
//         },
//         other => {
//             wrap_err!("fs.find expected string or FindQuery, got: {:?}", other)
//         }
//     }
// }

// #[allow(dead_code)]
// fn fs_find_file(luau: &Lua, path: LuaValue) -> LuaValueResult {
//     let path = match path {
//         LuaValue::String(path) => {
//             path.to_string_lossy()
//         },
//         other => {
//             return wrap_err!("fs.file expected string (path of the file to look for), got: {:#?}", other);
//         }
//     };

//     let metadata = match fs::metadata(&path) {
//         Ok(metadata) => metadata,
//         Err(err) => {
//             match err.kind() {
//                 io::ErrorKind::NotFound => {
//                     return Ok(LuaNil);
//                 },
//                 io::ErrorKind::PermissionDenied => {
//                     return wrap_err!("fs.file: attempted to find file at path '{}' but permission denied", path);
//                 },
//                 other => {
//                     return wrap_err!("fs.file: error getting metadata for file at path '{}': {:?}", path, other);
//                 }
//             }
//         }
//     };

//     if metadata.is_file() {
//         Ok(LuaValue::Table(
//             create_entry_table(luau, &path)?
//         ))
//     } else if metadata.is_dir() {
//         wrap_err!("fs.file: requested file at path '{}' is actually a directory", path)
//     } else {
//         unreachable!()
//     }
// }

// fn fs_find_dir(luau: &Lua, path: LuaValue) -> LuaValueResult {
//     let path = match path {
//         LuaValue::String(path) => {
//             path.to_string_lossy()
//         },
//         other => {
//             return wrap_err!("fs.dir expected string (path of the directory to look for), got: {:#?}", other);
//         }
//     };

//     let metadata = match fs::metadata(&path) {
//         Ok(metadata) => metadata,
//         Err(err) => {
//             match err.kind() {
//                 io::ErrorKind::NotFound => {
//                     return Ok(LuaNil);
//                 },
//                 io::ErrorKind::PermissionDenied => {
//                     return wrap_err!("fs.dir: attempted to find directory at path '{}' but permission denied", path);
//                 },
//                 other => {
//                     return wrap_err!("fs.dir: error getting metadata for directory at path '{}': {:?}", path, other);
//                 }
//             }
//         }
//     };

//     if metadata.is_dir() {
//         Ok(LuaValue::Table(
//             create_entry_table(luau, &path)?
//         ))
//     } else if metadata.is_file() {
//         wrap_err!("fs.dir: requested directory at path '{}' is actually a file", path)
//     } else {
//         unreachable!()
//     }
// }

pub fn fs_exists(_luau: &Lua, path: LuaValue) -> LuaValueResult {
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

// fn fs_create(luau: &Lua, new_options: LuaValue) -> LuaValueResult {
//     match new_options {
//         LuaValue::Table(options) => {
//             let entry_path = {
//                 if let LuaValue::String(file_path) = options.get("file")? {
//                     let writefile_options = TableBuilder::create(luau)?
//                         .with_value("path", file_path.to_owned())?
//                         .with_value("content", "")?
//                         .build_readonly()?;
//                     fs_writefile(luau, LuaValue::Table(writefile_options))?;
//                     file_path.to_str()?.to_string()
//                     // Ok(LuaNil)
//                 } else if let LuaValue::Table(file_options) = options.get("file")? {
//                    let file_name: LuaString = file_options.get("name")?;
//                    let file_content: LuaString = file_options.get("content")?;
//                    let writefile_options = TableBuilder::create(luau)?
//                         .with_value("path", file_name.to_owned())?
//                         .with_value("content", file_content)?
//                         .build_readonly()?;
//                     fs_writefile(luau, LuaValue::Table(writefile_options))?;
//                     file_name.to_str()?.to_string()
//                     // Ok(LuaNil)
//                 } else if let LuaValue::String(directory_path) = options.get("directory")? {
//                     let dir_path = directory_path.to_string_lossy().to_string();
//                     match fs::create_dir(&dir_path) {
//                         Ok(_) => dir_path,
//                         Err(err) => {
//                             match err.kind() {
//                                 io::ErrorKind::AlreadyExists => {
//                                     return wrap_err!("fs.create: error creating directory: directory '{}' already exists", dir_path);
//                                 },
//                                 _other => {
//                                     return wrap_err!("fs.create: error creating directory: {}", err);
//                                 }
//                             }
//                         }
//                     }
//                 } else if let LuaValue::Table(_tree) = options.get("directory")? {
//                     todo!()
//                 } else {
//                     return wrap_err!("fs.create expected {{file: string}} or {{file: {{name: string, content: string}}}}, but got something else");
//                 }
//             };
//             Ok(LuaValue::Table(create_entry_table(luau, &entry_path)?))
//         },
//         other => {
//             wrap_err!("fs.create expected to be called with table of type {{ file: string }} or {{ directory: string }}, got {:?}", other)
//         }
//     }
// }

fn fs_file_from(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let path = match value {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("fs.file.from(path) expected path to be a string, got: {:#?}", other);
        }
    };
    ok_table(file_entry::create(luau, &path))
}

pub fn create_filelib(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("from", fs_file_from)?
        .build_readonly()
}

fn fs_dir_from(luau: &Lua, value: LuaValue) -> LuaValueResult {
    let path = match value {
        LuaValue::String(path) => path.to_string_lossy(),
        other => {
            return wrap_err!("fs.dir.from(path) expected path to be a string, got: {:#?}", other);
        }
    };
    ok_table(directory_entry::create(luau, &path))
}

pub fn create_dirlib(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("from", fs_dir_from)?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    let std_fs = TableBuilder::create(luau)?
        .with_function("readfile", fs_readfile)?
        .with_function("readbytes", fs_readbytes)?
        .with_function("readlines", fs_readlines)?
        .with_function("writefile", fs_writefile)?
        .with_function("move", fs_move)?
        .with_function("removefile", fs_removefile) ?
        .with_function("listdir", fs_listdir)?
        .with_function("removetree", fs_removetree)?
        .with_function("entries", fs_entries)?
        // .with_function("find", fs_find)?
        // .with_function("file", fs_find_file)?
        // .with_function("dir", fs_find_dir)?
        // .with_function("create", fs_create)?
        .with_function("exists", fs_exists)?
        .with_value("path", pathlib::create(luau)?)?
        .with_value("file", create_filelib(luau)?)?
        .with_value("dir", create_dirlib(luau)?)?
        .build_readonly()?;

    Ok(std_fs)
}
