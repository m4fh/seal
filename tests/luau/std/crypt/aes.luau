local aes = require("@std/crypt/aes")

local key = aes.generatekey()

local ciphertext = aes.encrypt("meoweharhe", key)

local plaintext = aes.decrypt(ciphertext, key)
assert(plaintext == "meoweharhe", "aes plaintext doesn't match")


