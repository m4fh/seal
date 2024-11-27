--!strict
export type RunResult = ({
	ok: true,
	out: string,
	stdout: string,
	stderr: string,
} | {
	ok: false,
	err: string,
	stdout: string,
	stderr: string,
}) & {
	unwrap: (self: RunResult, default: any?) -> string
}

export type RunOptions = {
	program: string,
	args: { string }?,
	--- specify a shell to run the program with; otherwise runs it as a bare process with no shell
	shell: string?,
}

type SpawnOptionsConcurrent = {
	program: string,
	args: { string }?,
	concurrency: "Concurrent",
}

type SpawnHandleResult = {

}

type process = {
	--[=[
	Runs a program.

	Takes a RunOptions table:
	```luau
	type RunOptions = {
		program: string,
		--- optional args you want to pass
		args: { string }?,
		--- the name or path of the shell, omit to run without shell
		shell: string? 
	}
	```

	### Usage
	```luau
	local process = require("@std/process")
	local result = process.run {
		program = "lune",
		args = {"run", somefile},
	}
	if result.ok then
		print(result.stdout)
	end
	```
	]=]
	run: (options: RunOptions) -> RunResult,
	--[=[
	Launches a shell command in a child process.

	Uses the same shell you're using to run `seal` (so your aliases should available)
	### Usage
	```luau
	local process = require("@std/process")
	local file_stuff: {string} = process.shell("ls -l")
		:unwrap()
		:split("\n")
	print(file_stuff)
	```
	]=]
	shell: (command: string) -> RunResult,
	setexitcallback: ((number) -> ()) -> (),
	watch: (RunOptions) -> any,
	exit: (code: number?) -> never,
}

return {} :: process
