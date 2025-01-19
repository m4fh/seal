local hex = require("@std/serde/hex")
local password = require("@std/crypt/password")

local raw_test_password = "meowVeryComplexPassword2##@!#$"

local hashed_password: password.HashedPassword = password.hash(raw_test_password)
local hexified_hash = hex.encode(hashed_password.hash)
assert(#hexified_hash == 64, "why is hashed_password (PBKDF2_HMAC_SHA256) not hex encoding to a len 64 string?")

assert(hashed_password.hash and typeof(hashed_password.hash) == "buffer", "hashed_password.hash and is a buffer")
assert(hashed_password.salt and typeof(hashed_password.salt) == "buffer", "hashed_password.salt and is a buffer")

-- incorrect password is obviously verified to false
local not_real_password = "meowVerycomplexPassword2##@!#$" -- difference is lowercase c
assert(
	password.verify({
		raw_password = not_real_password,
		hashed_password = hashed_password :: password.HashedPassword,
	}) == false,
	"not_real_password == password, this is impossible and the world is ending"
)

-- correct password is obviously verified to true
local correct_password = "meowVeryComplexPassword2##@!#$"
assert(
	password.verify({
		raw_password = correct_password,
		hashed_password = hashed_password :: password.HashedPassword,
	}) == true,
	"password ~= password shouldn't be possible, issue with password verification?"
)