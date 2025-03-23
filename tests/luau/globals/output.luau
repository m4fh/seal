local meow = {
    fed = false,
    data = {
        ["/opt/idk/2"] = "safe",
    },
    cats = {
        Taz = 1,
        Nan = 2,
    },
    doer = function()
        
    end,
    [{ "i am a key" }] = true,
    [1.25] = 1,
    [vector.create(1, 2, 2312311.23)] = 12,
}
meow.a = meow
meow.cats.newcat = meow.cats
meow.meow = meow.cats.newcat
local buffy = buffer.create(32)
meow.b = buffy
local v = vector.create(0, 12, 2)
meow.v = v

local s, r = pcall(function()
    local fs = require("@std/fs")
    fs.readfile("we are an error. short days ago we were also error")
end)

meow.err = r

local user = newproxy()

meow.u = user

local coro = coroutine.create(function()

end)

meow.coro = coro

local seen_tables = {}
-- print(process_raw_values(meow, seen_tables))
-- print(`print(meow.err): {tostring(meow.err)}`)

local prettyvalues = {
    a = 1,
    [1] = "hi",
    [2] = "meow",
    [3] = "stasrtawf",
    b = "3",
    [1.25] = 4,
    ["hello world"] = true,
    ["emowtsartatf"] = {},
    ["/opt/cats/1"] = true,
    [buffer.create(12)] = "hi",
    location = vector.create(1, 2, 4),
}

meow.pretty = prettyvalues

print(prettyvalues)

local a = {}
a.b = { [a] = a }
print(a)

local cats = {}
cats.Taz = {
  name = "Taz",
  age = 12,
}

cats.Nan = {
  name = "Nanuk",
  age = 1,
}

cats.catlist = {
  cats.Taz, cats.Nan
}

cats.self = cats

print(cats)