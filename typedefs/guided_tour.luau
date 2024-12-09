--!nonstrict
--!nolint LocalUnused
--!nolint ImportUnused

-- hi!! welcome to sealey runtime ᶘ ᵔᴥᵔᶅ, here's some basics to get you set up 

-- there are > 1 ways to print things 
-- all prints will pretty print tables properly rendered/expanded/formatted
print({
	my = { table = 1 }
}) -- pretty prints in format, color, etc.
local string_says_hi = pp("hi") -- pretty print, but it returns
local debuggy = p("hi") -- debug print that prints the runtime/Rust mlua types; for example, this prints `String("hi")`
-- the latter two are also available in @std/io/output if you manage to shadow them for some reason

-- firstclass support for colorful
local colors = require("@std/colors") -- also known as "@std/io/colors"
local sealey = pp(`{colors.bold.cyan("ᶘ" )}{colors.blue("ᵔ")}{colors.red("ᴥ")}{colors.blue("ᵔ")}{colors.bold.cyan("ᶅ")}`)

-- standard libs are in @std/libname and can be required as so:
local fs = require("@std/fs")
local io = require("@std/io")
local env = require("@std/env")
local json = require("@std/json")
-- some libs are nested as so:
local input = require("@std/io/input")
local datetime = require("@std/time/datetime")

-- to read a file to string, use fs.readfile
local guided_tour = fs.readfile("./typedefs/guided_tour.luau")

-- to write a file, use fs.writefile
fs.writefile({
	path = "./my_path.txt",
	content = "huh",
	overwrite = true, -- overwrite is true by default and can be omitted
})

-- to get a file or directory's Entry, use fs.find (returns nil if not found)
local src_entry = fs.find("./my_path.txt") :: fs.FileEntry
local huh_text = src_entry:read()
-- to get a files xor directories, pass in a table:
local file_entry = fs.find { file = "./my_path.txt" }
local dir_entry = fs.find { directory = "./src" }

-- to delete that file we just made, use fs.remove
fs.remove { file = "./my_path.txt" }

-- to iterate over all entries in a directory, use fs.entries
for path, entry in fs.entries("./typedefs") do 
	if entry.type == "File" then
		if entry.extension == "luau" then
			local src = entry:read() -- reads to string
		end
	elseif entry.type == "Directory" then
		entry:list() -- list all entry filepath/filenames of the found dir
	end
end

-- to run/execute a new process, use process.run
local process = require("@std/process")
local files_res = process.run {
	program = "ls",
	shell = "zsh",
}

local files = pp(if files_res.ok then files_res.out else nil)

print(
	process.run({
		program = "echo 'hi im program'",
		shell = if env.os == "Linux" then "/bin/bash" 
				elseif env.os == "Windows" then "pwsh" 
				else "zsh"
	}).stdout
)

-- or more conveniently, process.shell handles the shell stuff for us:
local result = process.shell("ls ./*"):unwrap()

-- you can get tons of mileage outta running arbitrary processes
-- here, we're writing a luau script and running it with seal
fs.writefile {
	path = "./print_args.luau",
	content = "local args = require('@std/env').args; pp(args)"
}
local res = process.run {
	program = `{env.executable_path} ./print_args.luau who am i`,
	shell = if env.os == "Windows" then "pwsh" else "sh"
}
print(`args from spawned program: {res:unwrap()}`)

-- use the env lib to get cli args and learn cool stuff about the hosting process
local script_path = env.script_path
local current_executable --[[wheres my seal??]] = env.executable_path
print(`You are currently running {colors.bold.white(env.os)}`)

fs.remove { file = "./print_args.luau" }
assert(
	fs.find({ file = "./print_args.luau" }) == nil, "fs.remove failed to remove print_args.luau?"
)

-- many stdlib apis return a `Result` table that contains fields `ok`, some sort of output or result, and a function to :unwrap(which takes a default value):
local http = require("@std/net/http")
type RandomDog = {
	message: string,
	success: boolean,
}
local dog_result: RandomDog = http.get({ 
	url = "https://dog.ceo/api/breeds/image/random"
}):unwrap_json({
	message = "dog not found",
	success = "false",
})

local output = require("@std/io/output")

print(`Your dog of the day is: {output.format(dog_result)}`)