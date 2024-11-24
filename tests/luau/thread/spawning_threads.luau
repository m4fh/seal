local thread = require("@std/thread")
local time = require("@std/time")

local handle = thread.spawn({
	src = [[
		if channel then
			local time = require("@std/time")
			local datetime = time.datetime
			time.wait(1)
			local result = channel:read()
			print(result)
			local rn = datetime.now():format(datetime.common_formats["HH:MM (AM/PM)"])
			print(`time from thready: {rn}`)
			time.wait(1)
			channel:send({
				cats = "meow"
			})
		else
			error("concurrent thread run in main; this should be ran in a thread.spawn")
		end
	]]
})

print("running concurrently...")

handle:send({
	x = 1,
	y = 2
})

while time.wait(1) do
	local result = handle:read()
	if result then
		print("huh")
		print(result)
		break
	end
end

handle:join()