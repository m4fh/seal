local crypt = require("@std/crypt")

local aes = crypt.aes
local rsa = crypt.rsa

local AES_KEY = aes.generatekey()

local RSA_KEYS = rsa.generatekeys()
assert(RSA_KEYS.private:match("BEGIN PRIVATE"), "private key not privating")
assert(RSA_KEYS.public:match("BEGIN PUBLIC"), "public key not publicing")
