local hash = require("@std/crypt/hash")

local test_plaintext = "meow is the goat of course"

local hashed = hash.sha2(test_plaintext)

assert(typeof(hashed) == "buffer", "should be a buffer")