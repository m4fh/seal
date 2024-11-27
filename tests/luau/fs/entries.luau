local fs = require("@std/fs")

for path, entry in fs.entries(".") do
	if entry.type == "File" then
		-- print(`{entry.type}: {prettify(entry)}`)	
	end
end

local entry = fs.find("./Cargo.toml")

if entry then
    print("Cargo.toml found")
end

local _src = fs.find({directory = "./src"})

local ninexists = fs.find { directory = "hi" }
assert(ninexists == nil)

assert(fs.find{file = "Cargo.toml"} ~= nil)

-- handle cases
do
	try(function()
		fs.entries("")
		return nil
	end):expect_err("No such file or directory")
end
