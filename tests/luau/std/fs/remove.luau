local fs = require("@std/fs")

fs.writefile { 
	path = "./meow.txt",
	content = "meowewnmweo2",
}

print(fs.readfile("./meow.txt"))

fs.remove({ file = "./meow.txt" })
-- fs.remove {
-- 	file = "./meow.txt"
-- }