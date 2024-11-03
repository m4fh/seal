--[=[
The greatest filesystem standardlib (in luau) that's ever lived.
]=]
export type WriteFileOptions = {
	path: string,
	content: string,
	overwrite: boolean?,
}

export type Entry = (DirectoryEntry | FileEntry)

export type DirectoryEntry = {
	type: "Directory",
	path: string,
	list: (self: DirectoryEntry) -> {string},
	entries: (self: DirectoryEntry) -> {
		[string]: Entry
	},
	remove: (self: DirectoryEntry) -> (),
}

export type FileEntry = {
	type: "File",
	path: string,
	extension: string,
	read: (self: FileEntry) -> string,
	remove: (self: FileEntry) -> (),
}

export type fs = {
	readfile: (path: string) -> string,
	writefile: (WriteFileOptions) -> (),
	exists: (path: string) -> boolean,
	list: (directory: string) -> {string},
	entries: (directory: string) -> {
		[string]: Entry
	},
}

local fs = {}
--[=[
Reads a file from a given string `path`, relative to the current workspace directory.

Errors if `path` doesn't exist on the file system or if you don't have permission to access that path.
]=]
function fs.readfile(path: string): string
	return nil :: any
end

--[=[
Writes a file given `WriteFileOptions`, which looks like this:
```luau
local writefileoptions: WriteFileOptions = {
	path = "./my/path.txt",
	content = "some content i'm fine with overwriting ./my/path.txt with",
	overwrite = true,
}
```

WriteFileOptions.overwrite is false by default (and is optional)
]=]
function fs.writefile(options: WriteFileOptions): ()
	return nil :: any
end


--[=[
Returns a dictionary of paths: Entries you can use to further manipulate and access things on your filesystem.

`path` is is relative to your current directory.

Usage:
```luau
for path, entry in fs.entries(".") do
	if entry.type == "File" then
		local file_text = pp(entry:read())
		if entry.extension == "lua" then
			entry:remove()
			fs.writefile {
				path = path:gsub(".lua", ".luau"),
				content = file_text,
			}
		end
	elseif entry.type == "Directory" then
		local file_list = entry:list()
	end
end
```
]=]
function fs.entries(path: string): { [string]: Entry }
	return nil :: any
end


--[=[
Returns an array-like-table containing all the file names/paths in the provided directory.

`path` is relative to your current directory.

Usage:
```lua
local files = fs.list("./typedefs")
```
]=]
function fs.list(path: string): { string }
	return nil :: any
end

--[=[
Returns an `Entry` if a file/directory is found at the requested path, otherwise returns `nil`.

Usage:
```luau
-- find both files and directories
local dir = fs.find("./typedefs") 
if dir.type == "Directory" then
	print(dir:list())
end

-- find only files
local file = fs.find { file = "./typedefs" }
assert(file == nil) -- ./typedefs is a directory, not a file!!

-- find only directories
local dir = fs.find { directory = "./src" }
```
]=]
function fs.find(path: string | { directory: string} | { file: string }): Entry
	return nil :: any
end

export type RemoveOptions = RemoveFileOptions | RemoveDirectoryOptions

export type RemoveFileOptions = {
	file: string
}
export type RemoveDirectoryOptions = {
	directory: string,
	force: boolean?, --- this defaults to true
	remove_absolute_path: boolean?, -- only if this is specified and true will you be allowed to remove paths that start in /
}

--[=[
Removes a file/directory relative to your current directory.

Errors if the file/directory you're trying to remove already doesn't exist, or you're trying to remove a directory
but `RemoveDirectoryOptions.force` is `false`.

Usage:
```lua
if fs.find { file = "existence.jpeg" } then
	fs.remove { file = "existence.jpeg" }
end

-- delet everything (dont try this at home (unless you need to delete everything))
for path, entry in fs.entries(".") do 
	if entry.type == "File" then
		fs.remove { file = path }
	else
		fs.remove {
			directory = path,
			force = true, -- this also recurses; if false, fs.remove will only remove empty directories
		}
	end
end

-- if you need to, for some reason, and you're sure you've verified your input and that user input can't accidentally cause problems here, you can delete dirs by absolute path
fs.remove {
	directory = "/home/myusername/Projects/thing/src",
	force = true, -- this is enabled by default btw
	remove_absolute_path = true,
}

```
]=]
function fs.remove(options: RemoveOptions): ()
	return nil :: any
end

return fs