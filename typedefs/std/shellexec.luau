
export type ShellExecStatus = {
	ok: boolean,
	status_code: number,
}

--[=[
**[UNSAFE]**

Executes a shell command in the ***current*** shell process.

This can be useful for shell scripting purposes, but in 'real' programs please consider using `process.shell` instead.

### Usage:
```luau
local shex = require("@std/shellexec")
shex("cd ./somewhere; pwd")
```
As this uses 'unsafe' libc to execute a C command, it is not recommended for general-purpose use outside seal-as-a-shell-script-alternative.
]=]
local function shellexec(command: string): ShellExecStatus
	return nil :: any
end

return shellexec