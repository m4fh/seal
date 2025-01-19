local time = require("@std/time")

if channel then
	local message: { stuff : string }? = nil
	while time.wait(0.1) do
		message = channel:read()
		if message ~= nil then
			break
		end
	end
	if message ~= nil then
		channel:send(message.stuff .. " ok")
	end
	time.wait(0.25)
	local new_data = channel:read()
	if new_data then
		print(new_data)
	end
end
