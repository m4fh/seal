declare function pcall(protected_callback: any): (boolean, unknown)

declare function p<T...>(...: T...): T...

declare function pp<T...>(...: T...): T...

declare script: {
	entry_path: string,
	current_path: string,
	context: string?,
	required_files: {
		[string]: {
			name: string,
			src: string,
		},
	},
	src: string,
}

declare function try(f: () -> any): {
	ok: boolean,
	match: (self: any, matcher: {
		ok: (result: any?) -> any?,
		err: (err: any?) -> any?,
	}) -> any?,
	unwrap: (self: any, default: any?) -> any,
}