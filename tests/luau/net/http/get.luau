--!strict
--!nolint LocalShadow
local http = require("@std/net/http")

print(
	http.get {
		url = "https://jsonplaceholder.typicode.com/todos/1"
	}:unwrap()
)

print(
	http.get {
		url = "https://jsonplaceholder.typicode.com/posts",
		params = {
			userId = "1",
		}
	}:unwrap()
)

-- attempt to call :unwrap() with missing arg
local success, result = pcall(function()
	local should_actually_error = http.get {
		url = "",
		params = {
			catname = "Taz",
		}
	}:unwrap()
	return should_actually_error
end)
local result = tostring(result)

assert(
	success == false
	and result:match("without default argument")
	,
	"expected to match unwrap without default arg, didnt match"
)

-- attempt to call unwrap(nil)
local x = nil
local computed = x or nil

local success, result = pcall(function()
	http.get({
		url = "memes.com",
		body = {
			x = 2,
			y = 3,
		}
	}):unwrap(nil)
end)
local result = tostring(result)

assert(
	success == false
	and result:match("note: default argument provided but was nil")
	, 
	"that was supposed to break"
)