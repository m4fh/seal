--!strict
--!nolint LocalShadow
local fs = require("@std/fs")

local funny_story = "tests/data/funny_story.txt"

local buff = fs.readbytes(funny_story)
assert(buff ~= nil, "fs.readbytes default version broke?")

local buff = fs.readbytes(funny_story, 0, 16)
assert(buffer.tostring(buff) == "Once upon a time", "reading buff from bytes 0 to 16 isn't 'Once upon a time'?")

-- should fail with invalid parms
do
	try(function()
		fs.readbytes(funny_story, 0)
		return nil
	end):expect_err("if s is provided then f must also be provided")

	try(function()
		fs.readbytes(nil :: any)
		return nil
	end):expect_err("expected file path to be a string")

	try(function()
		fs.readbytes("./dontexisthuh")
		return nil
	end):expect_err("No such file")

	try(function()
		fs.readbytes("./src")
		return nil
	end):expect_err("Is a directory")

	-- negative should b impossible
	try(function()
		fs.readbytes("./tests/data/cats.json", -2, -5)
		return nil
	end):expect_err("start byte s must be >= 0!")

	try(function()
		fs.readbytes("./tests/data/cats.json", 0, -5)
		return nil
	end):expect_err("final byte f must be positive!")
end
