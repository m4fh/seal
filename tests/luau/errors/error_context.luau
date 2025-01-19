--!nonstrict
script.context = "sealion roulette"

local try = require("@std/testing/try")

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