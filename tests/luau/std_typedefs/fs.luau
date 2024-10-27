
export type WriteFileOptions = {
	path: string,
	content: string,
	overwrite: boolean?,
}

export type Entry = (DirectoryEntry | FileEntry) & {
	path: string
}

export type DirectoryEntry = {
	type: "Directory",
	list: (self: DirectoryEntry) -> {string},
	entries: (self: DirectoryEntry) -> {
		[string]: Entry
	}
}

export type FileEntry = {
	type: "File",
	extension: string,
	read: (self: FileEntry) -> string,
}

export type fs = {
	readfile: (path: string) -> string,
	writefile: (WriteFileOptions) -> (),
	exists: (path: string) -> boolean,
	list: (directory: string) -> {string},
	entries: (directory: string) -> {
		[string]: Entry
	}
}


return {} :: fs