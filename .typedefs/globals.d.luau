declare function pcall(protected_callback: any): (boolean, unknown)

declare function p<T...>(...: T...): T...

declare function pp<T...>(...: T...): T...

declare function warn<T...>(...: T...): nil
	
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