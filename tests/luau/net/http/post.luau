local net = require("@std/net")

local result = net.post {
	url = "https://jsonplaceholder.typicode.com/posts",
	body = {
		title = "meow",
		body = "somebodytexthmm",
		userId = 2,
	}
}

if result.ok then
	print(result:decode())
else
	print(result)
end
