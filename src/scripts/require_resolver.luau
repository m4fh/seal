--!optimize 2
--!nolint LocalShadow

local fs = require("@std/fs")
local json = require("@std/json")
local pathlib = require("@std/fs/path")

local function resolve_require_path(path: string): string | { err: string }
	-- strip .luau extension if user passed in a non-compliant path
	if string.sub(path, #path - 4) == ".luau" then
		path = string.sub(path, 1, #path - 5)
	end
	-- this function modifies path so let's keep a copy to report in errors
	local original_path_requested = path
	local requiring_file = "" do
		local debug_name: string = (debug :: any).info(3, "s") --[[ this should give us the 
			debug name (set by luau.load().set_name) for the chunk that called require(),
			in the format `[string "./src/somewhere.luau"]`
		]]
		requiring_file = string.sub(debug_name, 10, -3) -- grabs the part between `[string "` and `"]`
	end

	-- handle & expand directory aliases
	if string.sub(path, 1, 1) == "@" then
		-- we need to find the closest .luaurc
		local luaurc_path: string? do
			local current_path: string = requiring_file
			while true do
				local parent = pathlib.parent(current_path)
				if parent == nil then
					break
				else
					current_path = parent
					local possible_luaurc_path = pathlib.join(current_path, ".luaurc")
					if fs.exists(possible_luaurc_path) then
						luaurc_path = possible_luaurc_path
						break
					end
				end
			end
		end

		local alias = path:match(`^@([%w]+)/`) -- match the alias name `meow` in `@meow/somepath`
		if alias == nil then
			return { err = `unable to extract alias from '{original_path_requested}'; is it a valid alias?`}
		end
		if luaurc_path then
			local luaurc = json.readfile(luaurc_path :: string)
			local aliases = (luaurc.aliases or {}) :: { [string]: string }
			local alias_found = false
			local checked_aliases: { string } = {}
			for key, replacement_path in aliases do
				if string.find(path, `^@{key}/`) then
					path = string.gsub(path :: string, `@{key}/`, replacement_path)
					alias_found = true
					break
				end
				table.insert(checked_aliases, key)
			end
			if not alias_found then
				local err_message = `alias '{alias}' was not found in the .luaurc at '{luaurc_path}'\naliases found: {table.concat(checked_aliases, ", ")}`
				return { err = err_message }
			end
		else
			return { err = `alias '{alias}' in require '{original_path_requested}' could not be resolved as no '.luaurc' config files were found` }
		end
	-- handle relative paths
	elseif string.sub(path, 1, 1) == "." then
		local prefix_length = 1 -- length of the ./ or ../ we need to strip from path
		local current_path: string? do
			current_path = pathlib.parent(requiring_file) :: string?
			prefix_length += 1
			if current_path and string.sub(path, 1, 2) == ".." then
				current_path = pathlib.parent(current_path)
				prefix_length += 1
			end
		end

		if not current_path then
			return { err = `requested path {path} does not have a parent` }
		else
			local current_path = current_path :: string -- shadow b/c solver bug
			-- strip the ./ or ../ from path
			path = string.sub(path, prefix_length + 1)
			-- join the absolute path with the relative path
			path = pathlib.join(current_path, path)
		end
	else -- path doesn't start with @, .., or . and is invalid
		local err_message = 
			`Invalid require path format: Luau requires must start with a require alias (ex. "@alias/path") or relative path (ex. "./path" or "../path").\n` ..
			"Notes:\n"..
			"  - ending a require path with .luau is no longer recommended (but currently supported by seal)\n" ..
			`  - implicitly relative paths (ex. require("file.luau") without ./ or ../) are no longer allowed; see https://github.com/luau-lang/rfcs/pull/56"`
		return { err = err_message }
	end

	-- we need to look for an init.luau
	local path_dir = fs.dir(path)
	if path_dir then
		local current_path_with_init: string = pathlib.join(path, "init.luau")
		if fs.exists(current_path_with_init) then
			path = current_path_with_init :: string
		else
			return { err = `required directory '{path_dir.path}' doesn't contain an init.luau`}
		end
	else
		path ..= ".luau"
		if not fs.exists(path) then
			return { err = `file not found: {path}`}
		end
	end

	return path
end

return resolve_require_path