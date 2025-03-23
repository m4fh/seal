--!optimize 2
--!nolint LocalShadow
--[[
    doing highly recursive operations + lots of table iteration is faster directly in luau
    rather than through mlua's interop layer, which, among other things, has additional safety costs
    associated with iterating through tables
]]

local mlua = require("@interop/mlua")
local colors = require("@std/colors")
local output = require("@std/io/output")

local testvec = (vector.create :: any)(1, 2, 3, 4)
local s, f = pcall(function()
    return (testvec :: any).w == 4
end)

local VECTOR_FOUR_WIDE_MODE = if s then true else false

local function get_table_id(t: { [any]: any }): string
    if typeof(t) ~= "table" then
        error("[INTERNAL] output_process_values.get_table_id got passed something that isn't a table")
    end
    local tostringed_t = tostring(t)
    -- remove the `table: ` part of `table: 0x000062524e10a0d8`
    local tableid = string.sub(tostringed_t, 8, #tostringed_t)
    return tableid
end

local IDENTIFIER_PATTERN = "^[%a_][%w_]*$"

local function process_raw_values(
    value: any,
    seen_tables: { [string]: string }?,
    depth: number?
): string
    seen_tables = seen_tables or {}
    local result = ""
    local depth = depth or 0
    local current_indent = string.rep("    ", depth)
    if typeof(value) == "table" then
        local value = value :: { [any]: any }
        local table_id = get_table_id(value)
        local seen_name = seen_tables[table_id]
        if seen_name then
            result ..= "cyclic(" .. seen_name .. ")"
        else
            if depth == 0 then
                seen_tables[table_id] = "self" -- so we can recognize top level cyclic tables
            end
            result ..= "{\n"
            for key, val in value do
                result ..= current_indent
                result ..= "    "
                -- handle key stringification
                if typeof(key) == "string" then
                    -- we can omit brackets if key is an identifier-like key (like "meow1234" which is vs "cats are cool" which isn't)
                    if string.match(key, IDENTIFIER_PATTERN) then
                        result ..= key
                    else
                        result ..= "[\"" .. key .. "\"]"
                    end
                elseif typeof(key) == "number" then
                    if mlua.isint(key) then
                        result ..= "[" .. tostring(key) .. "]"
                    else -- is a LuaNumber
                        if math.round(key) == key then
                            result ..= "[" .. tostring(key) .. ".0]"
                        else
                            result ..= "[" .. tostring(key) .. "]"
                        end
                    end
                elseif typeof(key) == "table" then
                    local key = key :: { [any]: any }
                    local key_tableid = get_table_id(key)
                    local seen_key_name = seen_tables[key_tableid]
                    if seen_key_name then
                        result ..= "[cyclic(" .. seen_key_name .. ")]"
                    else
                        -- according to ariel et al, people probably want to see addresses in debug mode
                        local stringified_tableid = string.gsub(table_id, "0x0*", "")
                        result ..= "[table(0x" .. stringified_tableid .. ")]"
                        -- but pretty print should show full table
                    end
                else
                    result ..= "[" .. process_raw_values(key, seen_tables, depth + 1) .. "]"
                end
                result ..= " = "
                -- handle val stringification
                if typeof(val) == "table" then
                    local val = val :: { [any]: any }
                    local val_tableid = get_table_id(val)
                    local seen_val_name = seen_tables[val_tableid]
                    if seen_val_name then
                        result ..= "cyclic(" .. seen_val_name .. ")"
                    else
                        result ..= process_raw_values(val, seen_tables, depth + 1)
                    end
                elseif typeof(val) == "string" then -- no need for extra recursive call for common case
                    result ..= '"' .. val .. '"'
                else
                    result ..= process_raw_values(val, seen_tables, depth + 1)
                end
                result ..= ",\n"
            end
        end
        result ..= current_indent ..  "}"
    elseif typeof(value) == "string" then
        result ..= '"' .. value .. '"'
    elseif typeof(value) == "boolean" then
        result ..= tostring(value)
    elseif typeof(value) == "number" then
        if mlua.isint(value) then
            result ..= tostring(value)
        else
            if math.round(value) == value then
                result ..= tostring(value) .. ".0"
            else
                result ..= tostring(value)
            end
        end
    elseif typeof(value) == "function" then
        local function_id = tostring(value)
        function_id = string.gsub(function_id, "function: 0x0*", "")
        return "function<0x" .. function_id .. ">"
    elseif typeof(value) == "buffer" then
        local buffer_id = tostring(value)
        buffer_id = string.gsub(buffer_id, "buffer: 0x0*", "")
        return "buffer<0x" .. buffer_id .. ">"
    elseif type(value) == "vector" then
        local vec = "vector<" .. value.x .. ", " .. value.y .. ", " .. value.z
        if VECTOR_FOUR_WIDE_MODE then
            vec ..= ", " ..  value.z
        end
        vec ..= ">"
        return vec
    elseif mlua.iserror(value) then
        local stringified_error = tostring(value)
        local errsplit = string.split(stringified_error, "\n")
        local errmsglist = {}
        for _, m in errsplit do
            if string.find(m, "stack traceback") then
                break
            else
                table.insert(errmsglist, m)
            end
        end
        return "error<" .. output.unformat(table.concat(errmsglist)) .. ">"
    elseif type(value) == "userdata" then
        local typeof_value = typeof(value)
        if typeof_value ~= "userdata" then
            return "userdata<" .. typeof_value .. ">"
        else
            local userdata_id = tostring(value)
            userdata_id = string.gsub(userdata_id, "userdata: 0x0*", "")
            return "userdata<0x" .. userdata_id .. ">"
        end
    elseif type(value) == "nil" then
        return "nil"
    elseif type(value) == "thread" then
        local coro_id = tostring(value)
        coro_id = string.gsub(coro_id, "thread: 0x0*", "")
        return "coroutinethread<0x" .. coro_id .. ">"
    else
        return tostring(value)
    end
    return result
end

type TablePair = {
    key: any,
    val: any,
}

local function sort_table_pairs(unsorted_pairs: { TablePair }): { TablePair }
    local sorted_pairs: { TablePair } = table.create(#unsorted_pairs) :: { TablePair }
    local seen_indices: { [number]: boolean } = {} -- indices seen/visited in unsorted_pairs

    local numerical_pairs: { TablePair } = {} do
        for index, pair in unsorted_pairs do
            if type(pair.key) == "number" then
                seen_indices[index] = true
                table.insert(numerical_pairs, pair)
            end
        end
        table.sort(numerical_pairs, function(a: TablePair, b: TablePair)
            local isint_a = mlua.isint(a.key)
            local isint_b = mlua.isint(b.key)
            if isint_a and isint_b then
                return a.key < b.key
            elseif isint_a and not isint_b then
                return true
            elseif isint_b and not isint_a then
                return false
            else
                return a.key < b.key
            end
        end)
    end

    for _, v in numerical_pairs do
        table.insert(sorted_pairs, v)
    end

    local string_pairs: { TablePair } = {} do
        for index, pair in unsorted_pairs do
            if type(pair.key) == "string" then
                seen_indices[index] = true
                table.insert(string_pairs, pair)
            end
        end
        table.sort(string_pairs, function(a: TablePair, b: TablePair)
            local a_byte, b_byte = string.byte(a.key), string.byte(b.key)
            return a_byte < b_byte
        end)
    end

    for _, v in string_pairs do
        table.insert(sorted_pairs, v)
    end

    for index, pair in unsorted_pairs do
        if not seen_indices[index] then
            table.insert(sorted_pairs, pair)
        end
    end

    return sorted_pairs
end

local RESET = colors.codes.RESET
local RED = colors.codes.RED
local CYAN = colors.codes.CYAN
local MAGENTA = colors.codes.MAGENTA
local BLUE = colors.codes.BLUE
local BOLD_BLUE = colors.codes.BOLD_BLUE
local GREEN = colors.codes.GREEN
local YELLOW = colors.codes.YELLOW
local BLACK = colors.codes.BLACK

local ID_CATCHER_PATTERN = "[.]*0x0*([%d%w]+)"

local function process_pretty_values(
    value: any,
    seen_tables: { [{ [any]: any }]: string }?,
    depth: number?,
    parent_key: string?
): string
    seen_tables = seen_tables or {}
    local depth = depth or 0
    local current_indent = string.rep("    ", depth)
    if type(value) == "table" then
        local value = value :: { [any]: any }
        local result = "{\n"

        if depth == 0 then
            seen_tables[value] = "self" -- track top level cyclic tables
        end

        -- pretty printed tables should be printed in numerical, then alphabetical order (if possible)
        local unsorted_pairs: { TablePair } = {}
        for k, v in value do 
            table.insert(unsorted_pairs, { key = k, val = v }) 
        end

        local sorted_pairs: { TablePair } = sort_table_pairs(unsorted_pairs)
        if #sorted_pairs == 0 then
            return "{}" -- special case empty tables
        end
        for _, pair in sorted_pairs do
            local key, val = pair.key, pair.val
            result ..= current_indent .. "    "

            -- stringify keys
            local key_type = type(key)
            if key_type == "number" then
                result ..= "["
                       .. if mlua.isint(key) then BLUE .. tostring(key) .. RESET
                          else MAGENTA .. tostring(key) .. RESET
                result ..= "]" 
            elseif key_type == "string" then
                if string.match(key, IDENTIFIER_PATTERN) then
                    result ..= key
                else
                    result ..= `[{GREEN}"{key}"{RESET}]`
                end
            elseif key_type == "table" then
                local seen_name = seen_tables[key]
                if seen_name then
                    result ..= "[" .. colors.bold.white(seen_name) .. "]"
                elseif depth + 1 > 6 then
                    local tableid = string.match(tostring(key), ID_CATCHER_PATTERN)
                    result ..= "[" .. colors.cyan(`table<0x{tableid}>`) .. "]"
                else
                    result ..= "[" .. process_pretty_values(key, seen_tables, depth + 1) .. "]"
                end
            else
                result ..= `[{process_pretty_values(key, seen_tables, depth + 1)}]`
            end

            result ..= " = "
            -- stringify values
            if typeof(val) == "table" then
                local seen_name = seen_tables[val]
                if seen_name then
                    result ..= colors.codes.BOLD_YELLOW .. seen_name .. RESET
                elseif depth + 1 > 6 then
                    -- next(val) is nil if the table has no elements
                    if next(val :: any) == nil then
                        result ..= "{}"
                    else
                        result ..= "{" .. colors.codes.BOLD_WHITE .. "..." .. RESET .. "}"
                    end
                else
                    result ..= process_pretty_values(val, seen_tables, depth + 1)
                    seen_tables[val] = tostring(key)
                end
            else
                result ..= process_pretty_values(val, seen_tables, depth + 1)
            end
            result ..= ",\n"
        end
        result ..= current_indent .. "}"
        -- print(sorted_pairs)
        return result
    elseif type(value) == "number" then
        if not mlua.isint(value) and math.round(value) == value then
            return BLUE .. tostring(value) .. ".0" .. RESET
        else
            return BLUE .. tostring(value) .. RESET
        end
    elseif type(value) == "string" then
        return GREEN .. '"' .. value .. '"' .. RESET
    elseif type(value) == "boolean" then
        return YELLOW .. tostring(value) .. RESET
    elseif type(value) == "vector" then
        return `{RED}vector<{BOLD_BLUE}{value.x}{CYAN}, {BOLD_BLUE}{value.y}{CYAN}, {BOLD_BLUE}{value.z}{RESET}{RED}>{RESET}`
    elseif type(value) == "function" then
        local function_id = string.match(tostring(value), ID_CATCHER_PATTERN)
        local last_five = string.sub(function_id, #function_id - 4, # function_id)
        local first_part = string.sub(function_id, 1, #function_id - 5)
        if function_id then
            return RED .. "function<0x" .. first_part .. colors.bold.red(last_five) .. RESET .. RED .. ">" .. RESET
        else
            return RED .. "function" .. RESET
        end
    elseif type(value) == "buffer" then
        local buffer_id = string.match(tostring(value), ID_CATCHER_PATTERN)
        local last_five = string.sub(buffer_id, #buffer_id - 4, # buffer_id)
        local first_part = string.sub(buffer_id, 1, #buffer_id - 5)
        if buffer_id then
            return MAGENTA .. "buffer<0x" .. first_part .. colors.bold.magenta(last_five) .. RESET .. MAGENTA .. ">" .. RESET
        else
            return MAGENTA .. "buffer" .. RESET
        end
    elseif mlua.iserror(value) then
        local stringified_error = tostring(value)
        local errsplit = string.split(stringified_error, "\n")
        local errmsglist = {}
        for _, m in errsplit do
            if string.find(m, "stack traceback") then
                break
            else
                table.insert(errmsglist, m)
            end
        end
        return colors.codes.BOLD_RED .. "error" .. RESET .. RED .. "<" .. table.concat(errmsglist) .. ">" .. RESET
    elseif type(value) == "thread" then
        local thread_id = string.match(tostring(value), ID_CATCHER_PATTERN)
        if thread_id then
            return "corothread<" .. thread_id .. ">"
        else
            return "coroutinethread"
        end
    elseif type(value) == "userdata" then
        local typeof_userdata = typeof(value)
        if typeof_userdata ~= "userdata" then
            return colors.bold.magenta(typeof_userdata)
        else
            local userdata_id = string.match(tostring(value), ID_CATCHER_PATTERN)
            if userdata_id then
                return MAGENTA .. "userdata<0x" .. userdata_id .. ">" .. RESET
            else
                return MAGENTA .. "userdata" .. RESET
            end
        end
    else
        return tostring(value)
    end
end

-- local meow = {
--     fed = false,
--     data = {
--         ["/opt/idk/2"] = "safe",
--     },
--     cats = {
--         Taz = 1,
--         Nan = 2,
--     },
--     doer = function()
        
--     end,
--     [{ "i am a key" }] = true,
--     [1.25] = 1,
--     [vector.create(1, 2, 2312311.23)] = 12,
-- }
-- meow.a = meow
-- meow.cats.newcat = meow.cats
-- meow.meow = meow.cats.newcat
-- local buffy = buffer.create(32)
-- meow.b = buffy
-- local v = vector.create(0, 12, 2)
-- meow.v = v

-- local s, r = pcall(function()
--     local fs = require("@std/fs")
--     fs.readfile("we are an error. short days ago we were also error")
-- end)

-- meow.err = r
-- get_table_id(meow)

-- local user = newproxy()

-- meow.u = user

-- local coro = coroutine.create(function()

-- end)

-- meow.coro = coro

-- local seen_tables = {}
-- -- print(process_raw_values(meow, seen_tables))
-- -- print(`print(meow.err): {tostring(meow.err)}`)

-- local prettyvalues = {
--     a = 1,
--     [1] = "hi",
--     [2] = "meow",
--     [3] = "stasrtawf",
--     b = "3",
--     [1.25] = 4,
--     ["hello world"] = true,
--     ["emowtsartatf"] = {},
--     ["/opt/cats/1"] = true,
--     [buffer.create(12)] = "hi",
--     location = vector.create(1, 2, 4),
-- }

-- meow.pretty = prettyvalues

-- local output = require("@std/io/output")

-- output.write(process_pretty_values(meow, {}) .. "\n")

return {
    simple_print = process_raw_values,
    pretty_print = process_pretty_values,
}