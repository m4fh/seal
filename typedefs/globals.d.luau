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

declare channel: {
	send: <T>(self: any, data: T | string) -> (),
	read: <T>(self: any) -> T?,
	--- optional data passed into a thread upon thread.spawn
	data: any?,
}?

declare function try(protected_f: () -> any): {
	ok: boolean,
	match: (self: any, matcher: {
		ok: (result: any?) -> any?,
		err: (err: any?) -> any?,
	}) -> any?,
	unwrap: (self: any, default: any?) -> any,
}