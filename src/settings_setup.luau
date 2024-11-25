--!nolint LocalShadow
local std = require("@std")
local fs = require("@std/fs")

local vscode_dir = std.fs.find { directory = ".vscode" } :: fs.DirectoryEntry?
local new_settings = std.json.decode([[
	{
		"luau-lsp.require.directoryAliases": {
			"@std": "./typedefs/std/"
		},
		"luau-lsp.fflags.override": {
			"LuauSolverV2": "true",
			"LuauTypeSolverRelease": "651",
			"LuauNewSolverPopulateTableLocations": "true",
			"LuauNewSolverPrePopulateClasses": "true"
		},
		"luau-lsp.completion.imports.suggestRequires": true,
		"luau-lsp.types.definitionFiles": ["./typedefs/globals.d.luau"],
	}
]])

if not vscode_dir then
	fs.create { directory = ".vscode" }
end

local vscode_dir = fs.find { directory = ".vscode" } :: fs.DirectoryEntry
local settings_json = vscode_dir:find { file = "settings.json" } :: fs.FileEntry
if settings_json then
	local current_settings = std.json.decode(settings_json:read())
	for key, value in new_settings do
		if current_settings[key] then
			continue
		else
			current_settings[key] = value
		end
	end
	fs.writefile {
		path = ".vscode/settings.json",
		content = std.json.encode(current_settings)
	}
else
	vscode_dir:create {
		file = {
			name = "settings.json",
			content = std.json.encode(new_settings)
		}
	}
end

local src_dir = fs.find { directory = "src" }
if not src_dir then
	local src_dir = fs.create { directory = "src" } :: fs.DirectoryEntry
	src_dir:create {
		file = {
			name = "main.luau",
			content = `print("Hello world!!")`
		}
	}
end

local spec_dir = fs.find { directory = "spec" }
if not spec_dir then
	local spec_dir = fs.create { directory = "spec" } :: fs.DirectoryEntry
	spec_dir:create { 
		file = {
			name = "main.spec.luau",
			content = "-- insert tests here, if you want."
		}
	}
end
