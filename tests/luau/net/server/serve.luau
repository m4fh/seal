local server = require("@std/net/http/server")
local fs = require("@std/fs")
local json = require("@std/json")

print("starting seal server")

server.serve {
	address = "localhost",
	port = 4242,
	handler = function(info: server.ServeRequest)
		local response = {}
		if info.path == "/meow.json" then
			response.status_code = "200 OK"
			response.content_type = "json"
			response.body = json.encode {
				ok = true,
				says = "meow"
			}
		elseif info.path == "/" then
			local meow_page = fs.readfile("./tests/data/server-views/index.html")
			response.status_code = "200 OK"
			response.content_type = "html"
			response.body = meow_page
		elseif info.path == "/info" then
			local body = fs.readfile("./tests/data/server-views/info.html")
			response = {
				status_code = "200 OK",
				content_type = "html",
				body = body
			}
		elseif info.path == "/some-post" then
			response = {
				status_code = "200 OK",
				content_type = "application/json",
				body = json.encode {
					ok = true,
					recvbody = info.body,
				}
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