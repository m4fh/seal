local colors = require("@std/colors")
local state = require("./state")

local helper = {}

function helper.formatname(): string
	if state.data.name then
		return `@{colors.blue(state.data.name :: any)}`
	else
		error("state.data.name doesn't exist")
	end
end

function helper.changename(new_name: string)
	state.data.name = new_name
end

return helper