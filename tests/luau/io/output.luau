local output = require("@std/io/output")

local thing = {
	a = 2,
	b = 3,
	c = function(a: string) end,
	d = {
		s = 1,
		b = 2
	}
}

print(output.format(thing))

local dformatted = output["debug-format"](thing)

output["debug-print"](thing)