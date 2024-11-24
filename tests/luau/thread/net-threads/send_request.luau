local net = require("@std/net")
type Url = {
	url: string
}
if channel then
	local content = channel:read() :: Url?
	if content then
		local result = net.get({
			url = content.url
		})
		if result.ok then
			if result.body:match("^{") then
				channel:send(result:decode())
			else
				channel:send(result.body)
			end
		end
	end
else
	error("run me in a thread.spawn!!")
end
