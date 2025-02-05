local fs = require("@std/fs")
local env = require("@std/env")

local path = fs.path

local np = path.join("src", "main.luau")
if env.os ~= "Windows" then
	assert(np == "src/main.luau", "linux/unix simple paths broke")
else
	assert(np == [[src\main.luau]], "windows paths broke")
end

-- print(path.canonicalize(np))
if env.os ~= "Windows" then
	assert(path.absolutize(np):match("seal/src/main.luau"))
end

local cwd = path.child(path.absolutize("."))
assert(cwd == "seal", "cwd is not seal for some reason")

-- path.canonicalize can't canonicalize nonexistent path
local s, f = pcall(function()
	path.canonicalize("./existence.jpeg")
end)
assert(s == false, "canonicalize cannot succeed on ./existence.jpeg??")
assert(tostring(f):match("doesn't exist on the filesystem"))