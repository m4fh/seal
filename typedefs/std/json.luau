
type json = {
	encode: (t: any) -> string,
	decode: (json: string) -> any,
	readfile: (path: string) -> string,
	writefile: (WriteOptions: { path: string, content: { [any]: any } }) -> ()
}

return {} :: json