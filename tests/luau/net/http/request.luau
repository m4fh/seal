--!nolint LocalShadow
local net = require("@std/net")

local result = net.request {
	url = "https://jsonplaceholder.typicode.com/todos/1",
	method = "DELETE",
}
assert(result.ok, "issue deleting from placeholder endpoint")

local result = net.request {
	url = "https://jsonplaceholder.typicode.com/posts/1",
	method = "PATCH",
	body = {
		title = "hmmmm",
	}
}:unwrap()
assert(result.title == "hmmmm", "issue patching placeholder endpoint")
