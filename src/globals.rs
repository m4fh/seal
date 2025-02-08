use mlua::prelude::*;
use crate::*;

pub fn error(_luau: &Lua, error_value: LuaValue) -> LuaValueResult {
    wrap_err!("message: {:?}", error_value.to_string()?)
}

pub fn warn(luau: &Lua, warn_value: LuaValue) -> LuaValueResult {
    let formatted_text = std_io_output::format_output(luau, warn_value)?;
    println!("{}{}{}", colors::BOLD_YELLOW, formatted_text, colors::RESET);
    Ok(LuaNil)
}

const SEAL_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn set_globals(luau: &Lua) -> LuaValueResult {
    let globals: LuaTable = luau.globals();
    let luau_version: LuaString = globals.raw_get("_VERSION")?;
    globals.raw_set("require", luau.create_function(require::require)?)?;
    globals.raw_set("error", luau.create_function(error)?)?;	
    globals.raw_set("p", luau.create_function(std_io_output::debug_print)?)?;
    globals.raw_set("pp", luau.create_function(std_io_output::pretty_print_and_return)?)?;
    globals.raw_set("print", luau.create_function(std_io_output::pretty_print)?)?;
    globals.raw_set("warn", luau.create_function(warn)?)?;
    globals.raw_set("_VERSION", format!("seal {} | {}", SEAL_VERSION, luau_version.to_string_lossy()))?;
    globals.raw_set("_G", TableBuilder::create(luau)?
        // .with_metatable(TableBuilder::create(luau)?
        //     .with_value("__index", luau.globals())?
        //     .build_readonly()?
        // )?
        .build()?
    )?;
    globals.raw_set("_REQUIRE_CACHE", TableBuilder::create(luau)?.build()?)?;

    Ok(LuaNil)
}