--!nolint LocalShadow
--- configures a directory's .vscode and .luaurc settings for seal
-- doesn't handle spawning in .typedefs (that's handled in rust)
local fs = require("@std/fs")
local env = require("@std/env")
local json = require("@std/json")
local process = require("@std/process")

type DirectoryEntry = fs.DirectoryEntry
type FileEntry = fs.FileEntry

local sep = if env.os == "Windows" then [[\\]] else "/"

local global_definitions_filename = `.{sep}.typedefs{sep}globals.d.luau`

local default_vscode_settings = {
	["luau-lsp.fflags.enableNewSolver"] = true,
	["luau-lsp.platform.type"] = "standard",
	["luau-lsp.types.definitionFiles"] = { global_definitions_filename },
}

local default_luaurc = {
	languageMode = "strict",
	aliases = {
		std = `.{sep}.typedefs{sep}std{sep}`,
		interop = `.{sep}.typedefs{sep}interop{sep}`,
	} :: { [string]: string }
}

local function find_or_create_directory(path: string): DirectoryEntry
	local dir = fs.find { directory = path }
	if dir then
		return dir :: DirectoryEntry
	else
		return fs.create { directory = path } :: DirectoryEntry
	end
end

local function update_vscode_settings()
	local vscode_dir = find_or_create_directory(".vscode")
	
	local settings: { [string]: any } = {} do
		local settings_json = vscode_dir:find { file = "settings.json" } :: FileEntry?
		if settings_json then
			settings = json.decode(settings_json:read()) :: { [string]: any }
			settings["luau-lsp.fflags.enableNewSolver"] = settings["luau-lsp.fflags.enableNewSolver"] or default_vscode_settings["luau-lsp.fflags.enableNewSolver"]
			settings["luau-lsp.platform.type"] = settings["luau-lsp.platform.type"] or default_vscode_settings["luau-lsp.platform.type"]
			
			-- set or update definition files
			local definition_files = settings["luau-lsp.types.definitionFiles"] :: { string }?
			if definition_files then
				for _, file in default_vscode_settings["luau-lsp.types.definitionFiles"] do
					if not table.find(definition_files, file) then
						table.insert(definition_files, file)
					end
				end
			else
				definition_files = default_vscode_settings["luau-lsp.types.definitionFiles"]
			end

			settings["luau-lsp.types.definitionFiles"] = definition_files
		else
			settings = default_vscode_settings
		end
	end

	vscode_dir:create {
		file = {
			name = "settings.json",
			content = json.encode(settings) :: string,
		}
	}
end

local function update_luaurc()
	local luaurc = {}

	local luaurc_file = fs.file(".luaurc")
	if luaurc_file then
		luaurc = json.decode(luaurc_file:read())
		luaurc.languageMode = luaurc.languageMode or default_luaurc.languageMode
		local current_aliases: { [string]: string } = luaurc.aliases or {}
		for path, alias in default_luaurc.aliases do
			current_aliases[path] = alias
		end
		luaurc.aliases = current_aliases
	else
		luaurc = default_luaurc
	end

	fs.writefile { 
		path = ".luaurc",
		content = json.encode(luaurc) :: string,
	}
end

local function create_src_dir()
	local src_dir = fs.dir("src")
	if not src_dir then
		local src_dir = fs.create { 
			directory = "src"
		} :: DirectoryEntry
		src_dir:create {
			file = {
				name = "main.luau",
				content = 'print("Hello world!")',
			}
		}
	end
end

local function init_git_repository()
	if not fs.exists(".git") then
		local git_version_result = process.run { -- check if git exists on system
			program = "git",
			args = { "--version" },
		}
		if git_version_result.ok then
			local git_init_result = process.run {
				program = "git",
				args = { "init" },
			}
			if not git_init_result.ok then
				print("seal setup - error initializing git repository")
			end
		else
			print("seal setup - version control software `git` not found on your system - I recommend installing it! (https://git-scm.com/downloads)")
		end
	end
end

update_vscode_settings()
update_luaurc()
create_src_dir()
init_git_repository()
