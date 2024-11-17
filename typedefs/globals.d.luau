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

type TryParams = {
	try: (any?) -> any?,
	ok: (result: any?) -> any?,
	err: (err: any) -> any?,
}

declare function matcherr(params: TryParams): any