local net = {}

export type RequestConfig = {
	method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE",
	url: string,
	headers: { [string]: string }?,
	params: { [string]: string }?,
}

export type GetConfig = {
	url: string,
	headers: { [string]: string }?,
	--- Query parameters to append to the url string
	params: {
		[string]: string,
	}?
}

export type NetResponse = ({
	ok: true,
	body: string,
	--- decodes body to table, errors if body is invalid json or otherwise cannot be converted to table
	decode: (self: NetResponse) -> { [any]: any }
} | {
	ok: false,
	err: string,
}) & {
	--- decodes body as json or returns default value; errors if ok = false and default value not provided
	unwrap: (self: NetResponse, default: { [any]: any }?) -> { [any]: any }
}

--[=[
Makes an HTTP `GET` request.

## Usage
```lua
local response = net.get({
	url = "https://catfact.ninja/fact",
})
if response.ok then
	local raw_body = response.body
	local decoded_json_body = response:decode()
end

-- or with more features:

local cats = net.get {
	url = "my.cats.net/get",
	headers = {
		Authorization = someauth
	},
	params = {
		name = "Nanuk",
	},
}:unwrap()
```
]=]
function net.get(url: GetConfig): NetResponse
	return nil :: any
end

export type PostConfig = {
	url: string,
	headers: {
		[string]: string,
	}?,
	body: string | {
		[any]: any,
	},
}

--[=[
Makes an HTTP `POST` request.

## Usage
```lua
local response = net.post {
	url = "https://somejson.com/post",
	headers = {
		["API-KEY"] = api_key,
		-- note: Content-Type: application/json automatically handled when you pass a table as body!
	},
	body = {
		username = "hiItsMe",
	}
}

```#
]=]
function net.post(config: PostConfig): NetResponse
	return nil :: any
end

--[=[
Sends an HTTP request:

## Usage
```lua
local response = net.request({
	method = "PUT",
	url = "https://somewhere.net/api/put",
	body = somebody,
})

if response.ok then
	print(response:decode())
end
```
]=]
function net.request(config: RequestConfig): NetResponse
	return nil :: any
end

return net
