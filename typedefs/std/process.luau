--!strict
type RunResult = ({
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
	unwrap: (self: RunResult, default: any?) -> string?
}

type RunOptions = {
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
	run: (RunOptions) -> RunResult,
	spawn: (SpawnOptionsConcurrent) -> SpawnHandleResult,
	setexitcallback: ((number) -> ()) -> (),
	watch: (RunOptions) -> any,
	exit: (code: number?) -> never,
}

return {} :: process
