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
	encode: (t: any) -> string,
	decode: (json: string) -> any,
	readfile: (path: string) -> string,
	writefile: (WriteOptions: JsonWritefileOptions) -> ()
}

type JsonWritefileOptions = {
	path: string,
	content: { [any]: any }
}


return {} :: json