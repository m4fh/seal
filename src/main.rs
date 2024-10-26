use mlua::prelude::*;
use std::{fs, env, process, io};

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

fn require(luau: &Lua, path: String) -> LuaResult<LuaTable> {
    if path.starts_with("@std") {
        let std_fs = luau.create_table()?;
        std_fs.set("readfile", luau.create_function(read_file)?)?;

        match path.as_str() {
            "@std/fs" => Ok(std_fs),
            _ => {
                Err(LuaError::external(format!("unexpected standard library: {}", &path)))
            }
        }
    } else if path.starts_with("@") {
        Err(LuaError::external("invalid require path or not impl yet"))
    } else if path.starts_with("./") {
        Err(LuaError::external("invalid require path or not impl yet"))
    } else {
        Err(LuaError::external("invalid require path or not impl yet"))
    }
}

fn main() -> LuaResult<()> {
    let luau = Lua::new();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("bad usage");
        process::exit(1);
    }
    
    let luau_code: String = {
        let file_path = args[1].clone();
        if !file_path.ends_with(".luau") {
            eprintln!("file ext must be .luau");
            process::exit(1);
        } else if !fs::metadata(&file_path).is_ok() {
            eprintln!("File doesn't exist: {}", &file_path);
            process::exit(1);
        } else {
            fs::read_to_string(&file_path)?
        }
    };

    luau.globals().set("require", luau.create_function(require)?)?;

    match luau.load(luau_code).exec() {
        Ok(()) => Ok(()),
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }

}
