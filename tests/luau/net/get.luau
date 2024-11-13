--!strict
local net = require("@std/net")

print(
	net.get {
		url = "https://jsonplaceholder.typicode.com/todos/1"
	}:unwrap()
)

print(
	net.get {
		url = "https://jsonplaceholder.typicode.com/posts",
		params = {
			userId = "1",
		}
	}:unwrap()
)