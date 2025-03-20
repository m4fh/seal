--[=[
Interop with seal's underlying mlua layer
]=]
local mlua = {}

--- Returns `true` if `n` is an mlua Int, or `false` if it's an `mlua` Number
@checked
function mlua.isint(n: number): boolean
	return nil :: any
end

return mlua