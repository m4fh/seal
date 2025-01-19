--!strict

local fs = require("@std/fs")

local typedefs: fs.DirTree = {
	name = "typedefs",
	type = "Directory",
	content = {
		{
			name = "std",
			type = "Directory",
			content = {
				{
					name = "io",
					type = "Directory",
					content = {
						{
							name = "colors.luau",
							type = "File",
							content = ""
						}
					}
				}
			}
		}
	}
}
