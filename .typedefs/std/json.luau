--[=[
Easily manipulate JSON (JavaScript Object Notation) data.

### Usage
```luau
local json = require("@std/json")
-- read & deserialize json file to luau table
local data = json.readfile("./somefile.json")

-- Quick reference
type json = {
	encode: (t: any, options: EncodeOptions?) -> string,
	decode: (json: string) -> any,
	readfile: (path: string) -> string,
	writefile: (WriteOptions: JsonWritefileOptions) -> ()
}

type EncodeOptions = {
	--- with tabs/newlines/spaces for easy reading (default true)
	pretty: boolean?,
	--- sorted in alphanumeric order (default false)
	sorted: boolean?,
}

type JsonWritefileOptions = {
	path: string,
	content: { [any]: any }
}
```
]=]

type EncodeOptions = {
	--- with tabs/newlines/spaces for easy reading (default true)
	pretty: boolean?,
	--- sorted in alphanumeric order (default false)
	sorted: boolean?,
}

type json = {
	--- encodes json with newlines, readable
	encode: (t: any, options: EncodeOptions?) -> string,
	--- encodes json data without newlines, etc.
	encode_data: (t: any) -> string,
	decode: (json: string) -> any,
	readfile: (path: string) -> any,
	writefile: (WriteOptions: JsonWritefileOptions) -> (),
	writefile_data: (WriteOptions: JsonWritefileOptions) -> ()
}

type JsonWritefileOptions = {
	path: string,
	content: { [any]: any }
}


return {} :: json