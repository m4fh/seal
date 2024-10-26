type fs = {
		readfile: (string) -> string,
}

local fs = require("@std/fs") :: fs
local res = fs.readfile("./tests/data/funny_story.txt")

print(res)
