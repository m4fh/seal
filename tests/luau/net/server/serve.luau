local server = require("@std/net/server")
local fs = require("@std/fs")
local json = require("@std/json")

server.serve {
	address = "localhost",
	port = 4242,
	handler = function(info: server.ServeRequestInfo)
		local response = {}
		if info.path == "/meow.json" then
			response.status_code = "200 OK"
			response.response_type = "json"
			response.body = json.encode {
				ok = true,
				says = "meow"
			}
		elseif info.path == "/" then
			local meow_page = fs.readfile("./tests/data/server-views/index.html")
			response.status_code = "200 OK"
			response.response_type = "html"
			response.body = meow_page
		elseif info.path == "/info" then
			local body = fs.readfile("./tests/data/server-views/info.html")
			response = {
				status_code = "200 OK",
				response_type = "html",
				body = body
			}
		else
			response.status_code = "404 Not Found"
			response.response_type = "json"
			response.body = json.encode {
				ok = false,
			}
		end
		return response :: server.ServeResponse
	end
}