--!strict
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

type CatOwnerData = {
	reality: boolean,
	name: string,
	age: number,
	cats: {
		{ name: string, age: number }
	},
}

local t0 = os.clock()
local json_result = json.decode(got_data) :: CatOwnerData
print(os.clock() - t0)
print(json_result)

local fs = require("@std/fs")

json.writefile {
	path = "./tests/data/cats.json",
	content = json_result,
}

-- print(fs.entries("./tests/data/"))
local deviazecats = json.readfile("./tests/data/cats.json")
print(deviazecats)