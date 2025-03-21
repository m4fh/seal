
local a = {"2", "3", 4, { a = 3, b = 3.02 }, happy = false}
print(a)

local b = {
	happy = true,
}

b.b = b

print(b)

local fs = require("@std/fs")
local s, f = pcall(function()
	return fs.readfile("./startaftatafrst.")
end)
p(f)

print(package)