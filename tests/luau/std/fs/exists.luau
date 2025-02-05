local fs = require("@std/fs")
assert(fs.exists("./src") == true, "why does ./src not exist?")
assert(not fs.exists("./tfawtfat"), "fs.exists is broken")

