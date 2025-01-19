local io = require("@std/io")

print(io.colors.red("meow"))
print(io.input.get("whoami: "))
print(io.output["debug-print"]("meow"))

local x = io.output["print-and-return"]("thingsarethings")
assert(x == "thingsarethings" and x == pp("thingsarethings"))