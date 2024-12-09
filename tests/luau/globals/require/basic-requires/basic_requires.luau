local relative_data = require("./relative_data")
assert(relative_data.relative == "yes very")

local function meow()
	local relative_data_but_in_fn = require("./relative_data")
	assert(relative_data.relative == "yes very", "relative require in fn didnt work")
	assert(relative_data.callme() == "maybe", "can call function in relative require")
end

meow()

local parent_data = require("../some_data")
assert(parent_data.b and parent_data.b.cats)
print("basic requires work")