---- 时间 与 计时器 模块 -----

local F = {}

--- 协程版等待函数
--- @param ms number 毫秒
function F.sleep(ms)
	local co = coroutine.running()
	if not co then
		return
	end -- 如果不在协程里，直接返回

	local timer = vim.uv.new_timer()
	if not timer then
		return nil
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
end

return F
