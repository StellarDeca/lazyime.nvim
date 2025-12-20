----- 事件队列模块 -----
--- 对于同类型事件,只处理最新的事件
--- 旧的未处理事件直接舍弃

---@alias TaskFunction fun(params: table): any

---@class Task
---@field event string  -- 来自 autocmd 的 ev 表中的 ev 参数
---@field params table -- 任务函数参数
---@field task TaskFunction  -- 具体的业务逻辑函数

local F = {}

--- 任务队列存储
--- 键是事件名称 (string)，值是任务对象
---@type table<string, Task>
local task = {}

--- 队列事件顺序数组
--- 按照时间为从旧到新
--- 值为事件名称
---@type table<string>
local queue = {}

--- 防止竞争,加入竞争锁
--- work 运行中有且仅有一个实例
--- work 运行中时刻检查有无新 task
---@type boolean
local working = false

--- work 运行中有且仅有一个实例
--- work 运行中时刻检查有无新 task
--- work 会随着协程一同挂起与恢复
--- work 保证事项绝不遗漏 按顺序执行
function F.work()
	if working then
		return
	end
	working = true -- 加锁
	local co = coroutine.create(function()
		while #queue > 0 do
			local ev = table.remove(queue, 1)
			local task_ = task[ev]
			if task_ then
				-- 分配任务
				-- 大部分错误由 task 内部处理
				-- 这里仅仅捕获致命错误
				local ok, err = pcall(task_.task, task_.params)
				if not ok then
					vim.schedule(function()
						vim.notify("Could not handle error: " .. tostring(err), vim.log.levels.ERROR)
					end)
				end
			end
			task[ev] = nil
		end
		working = false -- 解锁
	end)
	coroutine.resume(co)
end

function F.wake_work()
	if not working then
		F.work()
	end
end

--- 推送任务
--- 同类型任务只保留最新
--- 不同类型任务按照时间顺序处理
---@param task_ Task
function F.push_task(task_)
	local ev = task_.event
	task[ev] = task_
	-- 重新排列任务队列
	-- 确保最新的任务始终在队尾
	local existing_index = nil
	for i, name in ipairs(queue) do
		if name == ev then
			existing_index = i
			break
		end
	end
	if existing_index then
		table.remove(queue, existing_index)
	end
	table.insert(queue, ev)
end

return F
