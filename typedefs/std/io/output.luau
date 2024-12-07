--[=[
Standard library for formatting and dealing with terminal output.

Usage:
```lua
local output = require("@std/io/output")

-- format a table with colors
local formatted = output.format({
	m = 2,
	d = 3
})

print(formatted)

-- use `p` after it got shadowed for some reason  
local p = getpvalue()
-- oh no i can't use debug print `p` anymore? debug-print to the rescue.
output["debug-print"]({
	p = p,
})
```
]=]
local output = {}

--- prettifies and colorizes any Luau values in the same way as `print`
function output.format(...: any): ...string
	return nil :: any
end

--- an uncolorized print for debugging that returns the printed values; same as the included global `p`.
output["debug-print"] = function(...: any): ...string
	return nil :: any
end

--- like `debug-print` but doesn't print the passed values (just formats them)
output["debug-format"] = function(...: any): ...string
	return nil :: any
end

--- regular colorized `print()` but it returns; same as included global `pp`
output["print-and-return"] = function(...: any): ...string
	return nil :: any
end

--- clears the terminal stdout, aka `cls` or `clear`
function output.clear(): nil
	return nil
end
return output