# seal, the cutest ~~runtime~~ for the [luau language](https://luau.org)

seal makes writing scripts and programs fun and easy, and runs them seally fast.

## Usage

Use `seal setup` to generate a new project in your current directory. This autogenerates a `.vscode/settings.json` to configure [Luau Language Server](https://github.com/JohnnyMorganz/luau-lsp), seal typedefs, `src` and `spec` directories, and a `.luaurc` for configuring Luau. Ideally, this means you should be able to just start writing code with no further configuration on your end!

To run a `.luau` file with seal, use `seal <filename>`. To evaluate code within seal (with fs/net/process libs already loaded in), use `seal eval '<string src>'`

Although seal provides some builtin globals, most features are in the standard library. You can import stdlibs like so:

```luau
local fs = require("@std/fs")
local net = require("@std/net")
local process = require("@std/process")
local colors = require("@std/colors") -- (the most important one tbh)

-- some libs are nested:
local input = require("@std/io/input")
```

If you're using VSCode and Luau Language Server, you should be able to see documentation, usage examples, and typedefs for each stdlib by hovering over their variable names in your editor. For convenience, all documentation is located in the `typedefs/*` directory generated alongside your project.

### Common tasks

- File wrangling

```luau
-- read a file to string
local file = fs.readfile("./path/to/file.ext") -- note, fs stdlib paths are relative to workspace/cwd root, not relative to file

-- write a file
fs.writefile {
    path = "./somewhere/filename.txt",
    content = "did you know seals can bark?",
} -- you can also use fs.create, which also allows you to create directories

-- iterate all files/dirs in a dir
for path, entry in fs.entries("./spec") do
    local result = process.shell(`seal {path}`):unwrap()
    entry:remove()
end

-- find an entry; entries provide many useful fields and methods to help you manipulate files & dirs
local very_file = fs.find { file = "./very_file" }
if very_file then
    -- read the file to string
    local text = very_file:read()
    -- read half the file
    local half_the_file = buffer.tostring(very_file:readbytes(0, very_file.size // 2))
    -- delet the file
    very_file:remove()
end

-- fs.find will use Luau type functions when they get stabilized and beta released, for now please annotate them types manually or use the Entry.type

```

- requests

```luau
local http = require("@std/net/http")

local response = http.get({
    url = "https://sealfinder.net/get",
})

local res = if response.ok then response:decode() else {
    location = "where the seals live"
}

-- you can also unwrap_json!
local res = http.get({
    url = "https://sealfinder.net/get",
}):unwrap_json({ -- all :unwrap() methods take an optional default argument
    location = "where the seals live"
})

```

- colorful

```luau
local colors = require("@std/colors")
local format = require("@std/io/format")

print(`{colors.red("oh no, anyways!:")}: {format(some_table)}`)
```

- cli args

```luau
local env = require("@std/env")

for _, arg in env.args do
    print(arg)
end
```

## Simple Structured Concurrency

seal is fully sans-tokio and async-less for performance, efficiency, and simplicity. Concurrency is hard. Concurrency + upvalues, even harder.

But, you want > 1 thing to happen at once nonetheless? seal provides access to Real Rust Threads with a relatively simple, low-level API. Each thread has its own Luau VM, which allows you to execute code concurrently. To send messages between threads, you can use the `:send()` and `:read()` methods located on both `channel`s (child threads) and `JoinHandle`s (parent threads), which seamlessly serialize, transmit, and deserialize Luau data tables between threads (VMs) for you!

Although this style of thread management is definitely less ergonomic than a `task` library or promise implementation, I hope this makes it more reliable and less prone to yields and UB, and is all-around a stable experience. I'll be working on improving this throughout seal's continual development, and I'm very open to a `task` lib equivalent getting contributed in for simple coroutine management.

```luau
-- parent.luau
local thread = require("@std/thread")

local handle = thread.spawn {
    path = "./child.luau",
    data = {
        url = "https://example.net",
    }
}
 -- do something else
local res = handle:read_await()

handle:join() -- don't forget to join your handles!
```

Child threads have a global `channel` exposed, which you can use to send data to the main thread:

```luau
-- child.luau
local http = require("@std/net/http")
if channel then
    local data = channel.data :: { url: string }
    local response = http.get({ url = data.url }):unwrap_json()
    channel:send(response)
end
```

Notes on concurrency: because each Luau value (a table, number, or function) is attached specifically to a single Luau VM, you cannot send values arbitrarily between VMs. To get around this, instead of allowing users to pass arbitrary upvalues between functions in different threads, seal exposes a low-level API for structured communication between threads, and handles serialization/deserialization of values on our end.
