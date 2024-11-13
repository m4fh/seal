local env = {}
--[=[
A stdlib to interact with the script's running environment.
```luau
type env = {
	--- a list of arguments passed to the program
	args: {string},
	--- your operating system
	os: "Windows" | "Linux" | "Android" | "MacOS" | "Other",
	--- the path of the executable
	executable_path: string,
	script_path: string,
	current_working_directory: string,
}
```
]=]
type env = {
	--- a list of arguments passed to the program
	args: {string},
	--- your operating system
	os: "Windows" | "Linux" | "Android" | "MacOS" | "Other",
	--- the path of the executable
	executable_path: string,
	script_path: string,
	current_working_directory: string,
}

return env :: env