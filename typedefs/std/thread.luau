local thread = {}

export type ThreadHandle = {
	join: (self: ThreadHandle) -> ()
}

function thread.spawn(spawn_options: {src: string} | {path: string}): ThreadHandle
	return nil :: any
end

return thread