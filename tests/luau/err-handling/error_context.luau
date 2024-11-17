local net = require("@std/net")

script.context = "sealion roulette"

local result = try(
	function()
		if math.random(10, 20) < 15 then
			error("bad luck bro")
		else
			return "survived"
		end
	end
):unwrap()

print(result)