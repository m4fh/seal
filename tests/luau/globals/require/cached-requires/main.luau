local colors = require("@std/colors")
local state = require("./state")

state.data.name = "seal"

local helper = require("./helper")

assert(helper.formatname() == `@{colors.blue("seal")}`, "state change 1 didnt work")
helper.changename("meow")
assert(helper.formatname() == `@{colors.blue("meow")}`, "state change 2 didnt work")
