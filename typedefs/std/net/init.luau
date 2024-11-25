local net = {}

export type NetResponse = ({
	ok: true,
	body: string,
	decode: (self: NetResponse) -> { [any]: any }
} | {
	ok: false,
	err: string,
}) & {
	--- decodes body as json or returns default value; errors if ok = false and default value not provided
	unwrap: (self: NetResponse, default: { [any]: any }?) -> { [any]: any }
}

export type GetConfig = {
	url: string,
	headers: string?,
	--- Query parameters to append to the url string
	params: {
		[string]: string,
	}?
}

--[=[
Makes an HTTP `GET` request.

## Usage:
```lua
local response = net.get("https://catfact.ninja/fact")
if response.ok then
	local raw_body = response.body
	local decoded_json_body = response:decode()
end
```
]=]
function net.get(url: GetConfig): NetResponse
	return nil :: any
end

export type PostConfig = {
	url: string,
	headers: {
		[string]: string,
	},
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

return net
