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
local io = require("@std/io")
local colors = require("@std/io/colors")
local json = require("@std/json")

-- to read a file to string, use fs.readfile
local guided_tour = fs.readfile("./typedefs/guided_tour.luau")

-- to write a file, use fs.writefile
fs.writefile({
	path = "./my_path.txt",
	content = "huh",
	overwrite = true, -- overwrite is true by default and can be omitted
})

-- to get a file or directory's Entry, use fs.find (it returns nil if not found)
local src_entry = fs.find("./my_path.txt")
local huh_text = pp(src_entry:read())


-- to delete that file we just made, use fs.remove
fs.remove { file = "./my_path.txt" }

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

-- to run/execute a new process, use process.run
local files_res = process.run {
	program = "ls",
	shell = "zsh",
}

local files = if files_res.ok then files_res.out else nil
print(files)

-- you can get tons of mileage outta running arbitrary processes

-- use the env lib to get cli args and learn cool stuff about the hosting process
local env = require("@std/env")
local basic_script = fs.writefile {
	path = "./print_args.luau",
	content = "local args = require('@std/env').args; pp(args)"
}
local res = process.run {
	program = `{env.executable_path} ./print_args.luau who am i`,
	shell = if env.os == "Windows" then "powershell" else "sh"
}
print(`args from spawned program: {if res.ok then res.out else nil}`)

fs.remove { file = "./print_args.luau" }
assert(fs.find { file = "./print_args.luau" } == nil)

