-- local colors = require("@std/colors")
local env = require("@std/env")

local cwd = env.current_working_directory

local function parse_traceback(traceback: string): string
	local new_traceback = traceback
		:gsub("%[string ", "[")
		:gsub("%s*%[C%]: in %?", "")
		:gsub(": in %?", ": in top level")
		:gsub(cwd, ".")

	return new_traceback
end

return parse_traceback