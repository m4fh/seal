--!nolint LocalUnused
--!nolint ImportUnused

-- hi!! welcome to runtime, here's some basics to get you set up 

-- there are > 1 ways to print things 
-- all prints will print tables properly rendered/expanded/formatted
print("hi") -- prints in color
local string_says_hi = pp("hi") -- print, but it returns
local debuggy = p("hi") -- debug print that prints the runtime/Rust mlua types; for example, this prints `String("hi")`
-- the latter two are also available in @std/io/output if you manage to shadow them for some reason

-- standard libs are in @std/libname and can be required as so:
local fs = require("@std/fs")
local process = require("@std/process")
local env = require("@std/env")
local io = require("@std/io")
local colors = require("@std/io/colors")
local json = require("@std/json")

-- to read a file to string, use fs.readfile
local guided_tour = fs.readfile("./typedefs/guided_tour.luau")

-- to write a file, use fs.writefile
fs.writefile({
	path = "./my_path.txt",
	content = "huh",
	overwrite = true, -- overwrite is false by default and can be omitted
})

-- to delete that file we just made, use fs.remove
fs.remove { file = "./my_path.txt"}

-- to get a file or directory's Entry, use fs.find (it returns nil if not found)
local src_entry = fs.find("./src")

-- to iterate over all entries in a directory, use fs.entries
for path, entry in fs.entries("./typedefs") do 
	if entry.type == "File" then
		if entry.extension == "luau" then
			print(entry:read()) -- read all .luau files
		end
	elseif entry.type == "Directory" then
		print(entry:list()) -- list all entry filepath/filenames of the found dir
	end
end

-- to run/spawn/execute a new process, use process.run
process.run {
	program = "ls"
}