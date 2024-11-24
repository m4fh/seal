local thread = {}

export type ThreadHandle = {
	join: (self: ThreadHandle) -> (),
	send: <D>(self: ThreadHandle, data: D | string) -> (),
	read: <D>(self: ThreadHandle) -> D?,
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