local fs = require("@std/fs")
local rsa = require("@std/crypt/rsa")

local keys = rsa.generatekeys()

fs.writefile { 
	path = "./tests/data/public.key",
	content = keys.public,
}

fs.writefile {
	path = "./tests/data/private.key",
	content = keys.private,
}

local encrypted = rsa.encrypt("whoami", keys.public)

local decrypted = rsa.decrypt(encrypted, keys.private)

assert(decrypted == "whoami", "rsa plaintext doesnt match")