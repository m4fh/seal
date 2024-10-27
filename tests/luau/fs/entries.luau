--!strict
local fs = require("@std/fs")
local prettify = require("@std/prettify")

for path, entry in fs.entries(".") do
	if entry.type == "File" then
		print(`{entry.type}: {prettify(entry)}`)
		
	end
end
