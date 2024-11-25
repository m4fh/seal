--!nolint LocalShadow
local fs = require("@std/fs")
local res = fs.readfile("./tests/data/funny_story.txt")
assert(res:match("Luau"), "fs.readfile did we read wrong file?")

-- we can read binary file?
local res = fs.readfile("./tests/data/nanuk.png")

-- make sure we get comparable results w/ fs.readbytes
local bytes = fs.readbytes("./tests/data/nanuk.png")
assert(#res == buffer.len(bytes), "fs.readfile and fs.readbytes giving different length?")

