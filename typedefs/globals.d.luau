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
	send: <D>(self: any, data: D | string) -> (),
	sendbytes: <D>(self: any, data: buffer) -> (),
	read: <D>(self: any) -> D?,
	read_await: <D>(self: any) -> D,
	readbytes: (self: any) -> buffer?,
	readbytes_await: (self: any) -> buffer,
	data: any?,
}?

declare function try(protected_f: () -> any): {
	ok: boolean,
	match: (self: any, matcher: {
		ok: (result: any?) -> any?,
		err: (err: any?) -> any?,
	}) -> any?,
	unwrap: (self: any, default: any?) -> any,
	expect_err: (self: any, handler: string | (err: unknown) -> any) -> any,
}