--!nonstrict
--!nolint LocalShadow
local thread = require("@std/thread")
local net = require("@std/net")
local format = require("@std/io/format")

local urls: {string} = {
	"https://meow.com",
	"https://google.com",
	"https://en.wikipedia.org/wiki/Cat",
	"https://catfact.ninja/fact",
	"https://play.typeracer.com",
	"https://jsonplaceholder.typicode.com/todos/1",
}

-- get synchronously
local s_t = os.clock()
local results = {}
for _, url in urls do
	local result = net.get({
		url = url,
	})
	if result.ok then
		results[url] = result.body
	end
end
print(`{os.clock() - s_t} seconds in serial`)

local s_t = os.clock()

local handles: { [string]: thread.ThreadHandle } = {}
local results: { [string]: string } = {}

for _, url in urls do
	local handle = thread.spawn({
		path = "./send_request.luau",
		data = {
			url = url,
		}
	})
	handles[url] = handle
end

local results_got = 0
while true do
	for url, handle in handles do
		local result = handle:read()
		if result then
			results[url] = result
			results_got += 1
		end
	end
	if results_got == #urls then
		break
	end
end

for url, handle in handles do
	handle:join()
end

print(`{os.clock() - s_t} seconds in parallel/synchronously with os threads :)`)

local results_by_len = {}
for url, result in results do
	results_by_len[url:gsub("https://", "")] = (
		if #result == 0 then `json len: {#format(result)} chars`
		else `html len: {#result} chars`
	)
end
print(results_by_len)