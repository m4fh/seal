local time = require("@std/time")
local datetime = time.datetime
-- assert(datetime == require("@std/time/datetime"))

local now = datetime.now()

local common_formats = datetime.common_formats

print(`the time is now {datetime.now():format(datetime.common_formats["HH:MM (AM/PM)"])}`)

local function nowfmt(format_name: string, format: string)
	return `{format_name}: {now:format(format)}`
end

for format_name, format_string in common_formats do 
	print(nowfmt(format_name, format_string))
end

local start_t = os.clock()
time.wait(2)
local elapsed = os.clock() - start_t
assert(math.round(elapsed) == 2, "wth why time not timing?")
