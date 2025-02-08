--!nolint ImportUnused
local fs = require("@std/fs")

local function relative()
	local t0 = os.clock()
	local relative_data = require("./relative_data")
	local t1 = os.clock()
	assert(relative_data.relative == "yes very", "relative require didnt work")
	assert((t1 - t0) < 0.0025, "basic relative require took too long")
end

relative()

local function parent()
	local parent_data = require("../some_data")
	assert(parent_data.b and parent_data.b.cats, "parent require didnt work")
end

parent()

local basic_requires_path = "./tests/luau/globals/require/basic-requires"

local function invalidfile()
	fs.writefile {
		path = `{basic_requires_path}/invalid_file.luau`,
		content = "local",
	}
	local s, f = pcall(function()
		(require)("./invalid_file")
	end)
	fs.remove {
		file = `{basic_requires_path}/invalid_file.luau`
	}
	assert(tostring(f):match("Expected identifier when parsing variable name"), "requiring an invalid file doesn't produce the expected error")
end

invalidfile()

local function circular()
	fs.writefile {
		path = `{basic_requires_path}/circular.luau`,
		content = 'return require("./basic_requires.luau")',
	}
	local s, f = pcall(function()
		(require)("./circular") -- best we can do is make sure it doesn't infinite yield (lune's behavior)
	end)
	if fs.file(`{basic_requires_path}/circular.luau`) then
		fs.remove {
			file = `{basic_requires_path}/circular.luau`
		}
	end
end

-- circular()

local function dirwithinit()
	if fs.dir(`{basic_requires_path}/canweinit`) then
		fs.remove({ directory = `{basic_requires_path}/canweinit`} :: any)
	end
	local init_dir = fs.create {
		directory = `{basic_requires_path}/canweinit`
	} :: fs.DirectoryEntry
	init_dir:create {
		file = {
			name = "init.luau",
			content = "return 'wecaninit'"
		}
	}

	local recv = (require)("./canweinit")
	assert(recv == "wecaninit", "we cannot require dir with init.luau?")
	fs.remove({ directory = `{basic_requires_path}/canweinit`} :: any)
end

dirwithinit()

local function showshelpermessage()
	local s, f = pcall(function()
		local relative_data = require("relative_data")
	end)
	assert(tostring(f):match("Invalid require path format"), "issue showing the long 'Invalid require path format' helper message")
end

showshelpermessage()

local function filenotfound()
	local s, f = pcall(function()
		local notfound = (require)("./notfound")
	end)
	assert(s == false, "requiring filenotfound was not supposed to succeed")
	assert(tostring(f):match("require: file not found:"), "not getting file not found when requiring a file that should not be found")
end

filenotfound()

local function filethaterrors()
	fs.writefile {
		path = `{basic_requires_path}/iwillbreak.luau`,
		content = 'error("hi")',
	}
	local s, f = pcall(function()
		(require)("./iwillbreak")
	end)
	assert(s == false, "iwillbreak.luau did not actually break?")
	assert(tostring(f):match('message: "hi"'), "iwillbreak broke with an unexpected error message")
	if fs.file(`{basic_requires_path}/iwillbreak.luau`) then
		fs.remove {
			file = `{basic_requires_path}/iwillbreak.luau`
		}
	end
end

filethaterrors()
