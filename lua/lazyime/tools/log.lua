----- 日志模块 -----
--- 上层只负责向日志队列中添加任务
--- 日志模块负责对日志任务进行处理
--- 日志模块不按照日志级别进行过滤
--- 日志模块根据事项频率进行过滤
--- 高频事项进行间隔记录

--- 错误样式
---@class Error
---@field name string 错误名
---@field error string 错误信息
---@field fatal boolean true-错误可恢复-false-不可恢复
---@field panic boolean true-程序应当崩溃

--- 日志输出样式
---@class LogMessage
---@field name string 事件名
---@field source string 事件模块名
---@field trace_id integer 调用链ID
---@field time number 事件时间
---@field level "INFO" | "ERROR" | "WARN" 日志级别
---@field context table? 插件全局缓存数据
---@field error Error? 错误信息
---@field info table? 详细信息

local F = {}
local iolib = require("lazyime.tools.iolib")

local Config = {
	max_time = 5 * 1000,
	log_path = iolib.get_log_path(),
	-- 每次tick最大日志处理数量
	max_tick_event_handle = 5,
}

local runtime = {
	--- 调用链ID
	--- 再同一个进程中
	--- 调用链ID唯一
	--- 相同任务的不同log调用链ID相同
	trace_id = 1,

	--- 日志事件缓存
	--- 保证相同调用链ID的日志的完整性
	--- 对于高频事项日志进行过滤
	---@class EventCache
	---@field last_time number 该类型事件上次接受“新任务”的时间
	---@field first_time number 该事件加入队列的时间
	---@field ready boolean 该事件是否准备好写入
	---@field traces table<integer, LogMessage[]> 存储该事件下不同 trace_id 的日志链
	---@type table<string, EventCache>
	log_cache = {},

	--- 日志系统崩溃后仅仅做一次通知
	--- 之后结束日志系统的运行
	---@type boolean
	stopped = false,
}

--- 日志级别
F.default = F.info
F.info = vim.log.levels.INFO
F.warn = vim.log.levels.WARN
F.error = vim.log.levels.ERROR

--- 返回错误
---@param name string
---@param error string
---@param fatal boolean
---@param panic boolean?
---@return Error
function F.make_error(name, error, fatal, panic)
	if panic == nil then
		panic = false
	end
	return {
		name = name,
		error = error,
		fatal = fatal,
		panic = panic,
	}
end

--- 获取调用链ID
---@return integer
function F.get_trace_id()
	runtime.trace_id = runtime.trace_id + 1
	return runtime.trace_id
end

--- 创建日志任务对象
---@param name string 事件名
---@param source string 事件模块名
---@param trace_id_ integer? 调用链ID
---@param context table? 插件全局缓存数据
---@param error Error? 错误信息
---@param info table? 详细信息
---@param level number? 日志级别默认INFO
---@return LogMessage
function F.make_log_task(name, source, trace_id_, context, error, info, level)
	local l
	if not level or level == F.info then
		l = "INFO"
	elseif level == F.warn then
		l = "WARN"
	elseif level == F.error then
		l = "ERROR"
	end
	local seconds, usec = vim.uv.gettimeofday()
	local ms = math.floor(usec / 1000)
	local formatted_time = os.date("%Y-%m-%d %H:%M:%S", seconds) .. ":" .. string.format("%03d", ms)
	return {
		name = name,
		source = source,
		trace_id = trace_id_ or 0,
		time = formatted_time,
		context = context,
		error = error,
		info = info,
		level = l,
	}
end

--- 向日志队列中添加任务
---@param log LogMessage
function F.push_log(log)
	if runtime.stopped then
		return
	end

	local now = vim.uv.now()
	local key = log.source .. log.name
	local tid = log.trace_id
	local cache = runtime.log_cache[key]

	if cache and cache.ready then
		-- 准备好写入的数据类型不在接受新的传入
		return
	end

	if cache == nil then
		--- 初始化
		runtime.log_cache[key] = {
			first_time = now,
			last_time = now,
			ready = false,
			traces = { tid = { log } },
		}
	elseif cache.traces[tid] == nil then
		--- 调用链ID不存在
		--- 比较同类型任务事件
		if now - cache.last_time >= Config.max_time then
			cache.first_time = now
			cache.last_time = now
			cache.traces[tid] = { log }
		end
		--- 高频任务舍弃
	elseif cache.traces[tid] then
		--- 存在相同调用链
		--- 追加任务
		cache.last_time = now
		table.insert(cache.traces[tid], log)
	end
end

--- 按照日期写入日志文件
---@param event string
function F.write_log(event)
	-- 收割这个事件类型
	local cache = runtime.log_cache[event]
	for _, logs in pairs(cache.traces) do
		for _, log in ipairs(logs) do
			local date = os.date("%Y-%m-%d", os.time())
			local path = string.format("%s/%s.log", Config.log_path, date)
			local ok, err = iolib.write(path, vim.inspect(log) .. "\n")
			if not ok then
				runtime.stopped = true
				local message = "Lazyime log mode panic!" .. vim.inspect(err)
				vim.schedule(function()
					vim.notify(message, F.warn)
				end)
			end
		end
	end
	--- 移除已做事项
	runtime.log_cache[event] = nil
end

--- 日志文件管理
--- 只保留一周的日志
--- 自动删除老旧日志
function F.logs_manage()
	iolib.gc_by_date(Config.log_path, 7, function(name)
		local y, m, d = name:match("(%d%d%d%d)%-(%d%d)%-(%d%d)")
		if not y then
			return nil
		end
		return {
			year = tonumber(y),
			month = tonumber(m),
			day = tonumber(d),
		}
	end)
end

--- 检查runtime.log_cache是否满足了写入条件
function F.log_tick()
	if runtime.stopped then
		return
	end
	local now = vim.uv.now()
	local i = Config.max_tick_event_handle
	for key, cache in pairs(runtime.log_cache) do
		if not cache.ready and cache.first_time and now - cache.first_time >= Config.max_time then
			cache.ready = true
			if i > 0 then
				F.write_log(key)
			end
			i = i - 1
		elseif cache.ready then
			if i > 0 then
				F.write_log(key)
			end
			i = i - 1
		end
	end
end

--- 启动 logger 服务
--- 每隔一段时间检查 log 事项清单
function F.run_logger()
	-- 清理多余日志文件
	F.logs_manage()

	-- 设置计时器
	local tick_interval = math.floor(Config.max_time / 2)

	local timer = vim.uv.new_timer()
	if not timer then
		return
	end
	timer:start(
		tick_interval,
		tick_interval,
		vim.schedule_wrap(function()
			if runtime.stopped then
				timer:stop()
				timer:close()
				return
			end
			F.log_tick()
		end)
	)
end

return F
