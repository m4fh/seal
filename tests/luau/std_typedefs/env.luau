type Env = {
	--- a list of arguments passed to the program
	args: {string},
	--- your operating system
	os: "Windows" | "Linux" | "Android" | "MacOS" | "SomeOtherOS",
	--- the path of the executable
	executable_path: string,
	script_path: string,
	current_working_directory: string,
}

return {} :: Env