local fs = require("@std/fs")

local view_path = fs.find { directory = "./tests/data/server-views" } :: fs.DirectoryEntry
local index_file = view_path:find { file =  "index.html" } :: fs.FileEntry

local read_normally = index_file:read()
local split_lines = read_normally:split("\n")

-- couldn't figure out how to easily implement FileEntry:readlines() w/ generalized iteration
index_file:readlines(function(line, line_number)
	if line_number == 1 then
		assert(line:match("<!DOCTYPE html>"), "line 1 doctype doesnt match")
	end
	assert(split_lines[line_number] == line, `line {line_number} doesn't match`)
end)
