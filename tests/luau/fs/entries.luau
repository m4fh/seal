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

local et = fs.find({directory = "./src"})
print(et)

local ninexists = fs.find { directory = "hi" }
assert(ninexists == nil)

assert(fs.find{file = "Cargo.toml"} ~= nil)

local s, e = pcall(function()
		fs.find{directory = "./src/main.rs"}
end)

assert(s == false and tostring(e):match("not a directory"))
