use crate::{table_helpers::TableBuilder, wrap_err, colors};
use crate::LuaValueResult;
use std::{thread, time::Duration};

use mlua::prelude::*;

fn time_wait(_luau: &Lua, seconds: LuaNumber) -> LuaValueResult {
    let millis = (seconds * 1000.0) as u64;
    let dur = Duration::from_millis(millis);
    thread::sleep(dur);
    Ok(LuaNil)
}

fn time_datetime_now(luau: &Lua, _: LuaValue) -> LuaValueResult {
    let now = chrono::Local::now();
    Ok(LuaValue::Table(
        TableBuilder::create(luau)?
            .with_value("unix_timestamp", now.timestamp())?
            .with_function("format", move |luau: &Lua, mut multivalue: LuaMultiValue| -> LuaValueResult {
                match multivalue.pop_back() {
                    Some(LuaValue::String(format_string)) => {
                        let format_string = format_string.to_str()?.to_string();
                        now.format(&format_string).to_string().into_lua(luau)
                    }, 
                    other => {
                        wrap_err!("DateTime.format expected format string to be a string, got: {:?}", other)
                    }
                }
            })?
            .build_readonly()?,
    ))
}

pub fn create_datetime(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("now", time_datetime_now)?
        .with_value("common_formats", TableBuilder::create(luau)?
            .with_value("ISO_8601", "%Y-%m-%d %H:%M")?
            .with_value("RFC_2822", "%a, %d %b %Y %H:%M:%S %z")?
            .with_value("RFC_3339", "%Y-%m-%dT%H:%M:%S%:z")?
            .with_value("SHORT_DATE", "%Y-%m-%d")?
            .with_value("SHORT_TIME", "%H:%M")?
            .with_value("FULL_DATE_TIME", "%A, %B %d, %Y %H:%M:%S")?
            // Common American formats
            .with_value("MM/DD/YYYY", "%m/%d/%Y")?
            .with_value("MM/DD/YYYY HH:MM (AM/PM)", "%m/%d/%Y %I:%M %p")?
            .with_value("MM/DD/YY", "%m/%d/%y")?
            .with_value("HH:MM (AM/PM)", "%I:%M %p")?
            .with_value("AMERICAN_FULL_DATE_TIME", "%A, %B %d, %Y %I:%M:%S %p")?
            .build_readonly()?
        )?
        .build_readonly()
}

pub fn create(luau: &Lua) -> LuaResult<LuaTable> {
    TableBuilder::create(luau)?
        .with_function("wait", time_wait)?
        .with_value("datetime", create_datetime(luau)?)?
        .build_readonly()
}
