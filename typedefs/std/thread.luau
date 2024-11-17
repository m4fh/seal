local thread = {}

export type ThreadHandle = {
	join: (self: ThreadHandle) -> ()
}

function thread.spawn(spawn_options: {src: string} | {path: string}): ThreadHandle
	return nil :: any
end

--[=[
Literally the same as `time.wait`, except in milliseconds.
]=]
function thread.sleep(milliseconds: number): ()
	return nil :: any
end

return thread