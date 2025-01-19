local module = require("./another_module")

local function main()
	script.context = "rfing"
	module.rf()
	return module.execute()
end

main()