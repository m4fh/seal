local hex = require("@std/serde/hex")

local random_buffy = buffer.create(4)
local original_number = 2131

buffer.writef32(random_buffy, 0, original_number)

local data = hex.encode(random_buffy)
assert(#data == 8, "f32 should be encoded to a string 8 characters long")
local decoded = hex.decode(data)
local decoded_number = buffer.readf32(decoded, 0)
assert(decoded_number == original_number, "decoded number not original number")