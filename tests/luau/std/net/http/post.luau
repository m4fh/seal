local http = require("@std/net/http")

local result = http.post {
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
