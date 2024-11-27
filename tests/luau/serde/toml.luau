local toml = require("@std/serde/toml")

local stuff = toml.readfile("./Cargo.toml")
assert(stuff.dependencies ~= nil, "toml.readfile Cargo.toml does not have dependencies? Are we in the right project?")