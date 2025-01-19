local thread = {}

export type ThreadHandle = {
	join: (self: ThreadHandle) -> (),
	send: <D>(self: ThreadHandle, data: D | string) -> (),
	sendbytes: <D>(self: ThreadHandle, data: buffer) -> (),
	read: <D>(self: ThreadHandle) -> D?,
	read_await: <D>(self: ThreadHandle) -> D,
	readbytes: (self: ThreadHandle) -> buffer?,
	readbytes_await: (self: ThreadHandle) -> buffer,
}

function thread.spawn(spawn_options: {src: string, data: any?} | {path: string, data: any?}): ThreadHandle
	return nil :: any
end

--[=[
Literally the same as `time.wait`, except in milliseconds.
]=]
function thread.sleep(milliseconds: number): true
	return nil :: any
end

return thread