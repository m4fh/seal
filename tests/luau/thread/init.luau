local thread = require("@std/thread")

local handle = thread.spawn({
	path = "./conc_1.luau"
})

print("first")

handle:join()

print("two")