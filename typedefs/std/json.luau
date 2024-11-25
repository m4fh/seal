--[=[
Easily manipulate JSON (JavaScript Object Notation) data.

### Usage
```luau
local json = require("@std/json")
-- read & deserialize json file to luau table
local data = json.readfile("./somefile.json")

-- Quick reference
type json = {
	encode: (t: any) -> string,
	decode: (json: string) -> any,
	readfile: (path: string) -> string,
	writefile: (WriteOptions: JsonWritefileOptions) -> ()
}

type JsonWritefileOptions = {
	path: string,
	content: { [any]: any }
}
```
]=]
type json = {
	--- encodes json with newlines, readable
	encode: (t: any) -> string,
	--- encodes json data without newlines, etc.
	encode_data: (t: any) -> string,
	decode: (json: string) -> any,
	readfile: (path: string) -> string,
	writefile: (WriteOptions: JsonWritefileOptions) -> (),
	writefile_data: (WriteOptions: JsonWritefileOptions) -> ()
}

type JsonWritefileOptions = {
	path: string,
	content: { [any]: any }
}


return {} :: json