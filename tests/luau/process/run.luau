--!strict
local process = require("@std/process")

local res = process.run {
	program = "cd ./src; pwd",
	shell = "zsh"
}

print(
	if res.ok then res.out else res.err
)
