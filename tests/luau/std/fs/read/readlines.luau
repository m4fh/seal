local fs = require("@std/fs")

local view_path = fs.find { directory = "./tests/data/server-views" } :: fs.DirectoryEntry
local index_file = view_path:find { file =  "index.html" } :: fs.FileEntry

local read_normally = index_file:read()
local split_lines = read_normally:split("\n")

for line_number, line in index_file:readlines() do
	if line_number == 1 then
		assert(line:match("<!DOCTYPE html>"), "line 1 doctype doesnt match")
	end
	assert(split_lines[line_number] == line, `line {line_number} doesn't match`)
end
