--!strict
local fs = require("@std/fs")

if fs.find("./tests/data/meow") then
	fs.remove({ directory = "./tests/data/meow" } :: any)
end
local dir = fs.create {
	directory = "./tests/data/ignore/meow"
} :: fs.DirectoryEntry

for i = 1, 10 do
	dir:create {
		file = {
			name = `meow{i}.log`,
			content = `meow and {i}`
		}
	}
end

if dir:find({ directory = "inside" }) then
else
	(dir :: fs.DirectoryEntry):create({ directory = "inside" })
end


for path, entry in dir:entries() do
	local n = tonumber(path:match("meow([%d])"))
	if n and n % 2 == 0 then
		-- print(path)
		-- print(path, dir.path)
		local d = fs.find { directory = "./tests/data/ignore/meow/inside" }
		if d then
			fs.move(path, `{d.path}/meow{n}.log`)
		end
		-- fs.move(path, dir.path .. "/inside")
	end
end
