local json = require("@std/json")

local x = {
		c = 5,
		b = "meow",
}

print(json.encode(x))

local unencodable = {
		name = "deviaze",
		says = function()
				return "hi"
		end
}

local s, e = pcall(function()
		json.encode(unencodable)
end)

assert(s == false and tostring(e):match("cannot serialize <function>"))

local got_data = [[
{
  "name": "deviaze", 
  "age": 23, 
  "reality": false, 
  "cats": [{
    "name": "Taz",
	"age": 8
  }, {
    "name": "Nanuk",
	"age": 0.95
  }]
}
]]

local json_result = json.decode(got_data)
print(json_result)
