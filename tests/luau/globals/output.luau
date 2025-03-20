
local a = {"2", "3", 4, { a = 3, b = 3.02 }, happy = false}
print(a)

local b = {
	happy = true,
}

b.b = b

print(b)