--!nolint LocalShadow

local fs = require("@std/fs")
local testing = require("@std/testing")

local gitignore = fs.find { file = ".gitignore" }
assert(gitignore ~= nil, "fs.find { file: string } where is .gitignore")

local gitignore = fs.file(".gitignore")
assert(gitignore ~= nil, "where is .gitignore")

local dot_git = fs.dir(".git")
assert(dot_git ~= nil, "where is .git")

-- using fs.file on a dir errors
testing.try(function()
	local notactuallyfile = fs.file("./src")
	return nil
end):expect_err("'./src' is actually a directory")

-- using fs.dir on a file errors
testing.try(function()
	local notactuallydir = fs.dir("./.gitignore")
	return nil
end):expect_err("'./.gitignore' is actually a file")

-- TODO: make sure below errors
-- local gitignore = fs.find { directory = ".gitignore "}