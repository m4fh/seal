local process = require("@std/process")
local env = require("@std/env")

local run_with_shell = if env.os == "Linux" or env.os == "Android" then "sh" else "pwsh"

local res = process.run {
	program = "cd ./src; pwd",
	shell = run_with_shell
}

assert(res.ok == true, "result failed")

local fail_res = process.run {
	program = "idontexist",
	shell = run_with_shell
}

assert(fail_res.ok == false, "why do you have a program named `idontexist` on your system?")

local success, result = pcall(function()
	fail_res:unwrap()
end)
assert(tostring(result):match("without a default value!"), "RunResult:unwrap() broke?")
