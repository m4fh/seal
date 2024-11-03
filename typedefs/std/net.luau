local net = {}

export type GetResponse = {
	ok: true,
	body: string,
	decode: (self: GetResponse) -> { [any]: any }
} | {
	ok: false,
	err: string,
}

export type GetConfig = {
	url: string,
	headers: string?,
	parameters: {
		[string]: string,
	}?
}

--[=[
Makes an HTTP `GET` request.

Usage:
```lua
local response = net.get("https://catfact.ninja/fact")
if response.ok then
	local raw_body = response.body
	local decoded_json_body = response:decode()
end
```
]=]
function net.get(url: string | GetConfig): GetResponse
	return nil :: any
end

return net
