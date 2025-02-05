local fs = require("@std/fs")

for path, entry in fs.entries(".") do
	if entry.type == "File" then
		assert(entry.extension, "where extension?")
		assert(entry.name ~= nil, "where entry.name?")
		local content = (entry :: fs.FileEntry):read()
		assert(typeof(content) == "string", "file content is not string?")
	else
		for nested_path, nested_entry in entry:entries() do
		end
	end
end

local entry = fs.find("./Cargo.toml")
assert(entry ~= nil and entry.type == "File", "Cargo.toml not found?")

local src = fs.find({directory = "./src"})
assert(src ~= nil, "no src?")
assert(src.name == "src", "src name not src?")
assert(src.type == "Directory", "src not a directory???")

local ninexists = fs.find { directory = "hi" }
assert(ninexists == nil)

assert(fs.find{file = "Cargo.toml"} ~= nil)

local try = require("@std/testing/try")

-- handle cases
do
	try(function()
		fs.entries("")
		return nil
	end):expect_err("No such file or directory")
end
