local relative_data = require("./relative_data")
assert(relative_data.relative == "yes very")

local parent_data = require("../some_data")
print(parent_data)