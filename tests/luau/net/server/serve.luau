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
			response.response_type = "JSON"
			response.body = json.encode {
				ok = true,
				says = "meow"
			}
		elseif info.path == "/meow.html" then
			response.status_code = "200 OK"
			local meow_page = fs.readfile("./tests/luau/net/server/meow-page.html")
			response.response_type = "HTML"
			response.body = meow_page
		elseif info.path == "/nanuk.png" then
			response.status_code = "200 OK" 
			local image_buff = fs.readbytes("./tests/luau/net/server/nanuk.png")
			response.response_type = "image/png" 
			response.body = image_buff
		else
			response.status_code = "404 Not Found"
			response.response_type = "JSON"
			response.body = json.encode {
				ok = false,
			}
		end
		return response :: server.ServeResponse
	end
}