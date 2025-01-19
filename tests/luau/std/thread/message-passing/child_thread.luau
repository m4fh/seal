local time = require("@std/time")

if channel then
	print("we in thread bois")

	while time.wait(1) do
		local data = channel:read()
		if data then
			local d = data :: {
				t: number,
				content: string,
			}

			if d.content == "break" then
				break
			end

			channel:send {
				t = d.t,
				content = d.content:gsub("well", "then")
			}
			
		end
	end
end