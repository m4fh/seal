local time = require("@std/time")

local start_t = os.clock()
time.wait(2)
local elapsed = os.clock() - start_t
assert(math.round(elapsed) == 2, "wth why is time not timing?")