---- 时间 与 计时器 模块 -----

local F = {}

--- 协程版等待函数
---@param ms number 毫秒
---@return true? ok, Error? error
function F.sleep(ms)
	local co = coroutine.running()
	if not co then
		return nil,
			{
				name = "SleepOutsideCoroutine",
				error = "time.sleep must be called inside a coroutine",
				fatal = true,
			}
	end

	local timer = vim.uv.new_timer()
	if not timer then
		return nil, {
			name = "TimerCreateFailed",
			error = "vim.uv.new_timer() returned nil",
			fatal = true,
		}
	end

	timer:start(ms, 0, function()
		timer:stop()
		timer:close()
		-- 时间到了，唤醒协程
		vim.schedule(function()
			coroutine.resume(co)
		end)
	end)

	-- 挂起当前协程
	coroutine.yield()
	return true, nil
end

return F
