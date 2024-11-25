local fs = require("@std/fs")
local base64 = require("@std/serde/base64")

local nanuk_buffy = fs.readbytes("./tests/data/nanuk.png")

local sensible_nanuk = base64.encode(nanuk_buffy)
print(sensible_nanuk)

local meow_page = fs.readfile("./tests/data/server-views/index.html")
local nanuk_encoded_64 = base64.encode(fs.readbytes("./tests/data/nanuk.png"))
meow_page = meow_page:gsub("!!REPLACEME!!", `src="data:image/png;base64,{nanuk_encoded_64}"`)
fs.writefile {
	path = "./tests/data/server-views/index-nanuk-embedded.html",
	content = meow_page,
}