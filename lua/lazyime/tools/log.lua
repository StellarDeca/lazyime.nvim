----- 日志模块 -----
--- 日志文件按照时间顺序排列
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

--- 日志事务样式
--- 日至事务由多个ID相同的Log组成
--- 每次push log仅仅推送了事务中的一部分
--- 某个事务要么全盘接受要么全部丢弃
---@class TraceEvent
---@field trace_id integer  调用链ID
---@field logs LogMessage[]  调用链事项数组
---@field push_time number 首次push的时间
---@field already boolean 事务是否可以开始记录

--- 日志输出样式
---@class LogMessage
---@field name string 普通事件名
---@field source string 事件模块名
---@field trace integer 调用链ID
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
}

local runtime = {
	--- 调用链ID
	--- 再同一个进程中
	--- 唯一标识一次调用链路
	--- 事项是逐步处理的，不需要考虑并发
	---@type { id: integer, type: string? }
	trace = {
		id = 1,
		type = nil,
	},

	--- 日志事件缓存
	--- 每个调用链路一条记录
	--- 按照调用链路类型去存储
	---@type table<string, TraceEvent>
	event_cache = {},

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

--- 创建日志任务对象
---@param name string 事件名
---@param source string 事件模块名
---@param context table? 插件全局缓存数据
---@param error Error? 错误信息
---@param info table? 详细信息
---@param level number? 日志级别默认INFO
---@return LogMessage
function F.make_log_task(name, source, context, error, info, level)
	if not level then
		level = F.default
	end
	local seconds, usec = vim.uv.gettimeofday()
	return {
		name = name,
		source = source,
		trace = runtime.trace.id,
		time = (seconds * 1000) + math.floor(usec / 1000), --- 转换为 毫秒ms
		context = context,
		error = error,
		info = info,
		level = level,
	}
end

--- 获取新的调用链ID
function F.next_trace_id()
	runtime.trace.id = runtime.trace.id + 1
end

--- 为了区分调用链路的类型
--- 在调用链路的起始使用nvim event name作为类型标识符
--- log缓存调用链路类型
--- 后续的log push无需传入类型参数直接获取当前调用链路类型
--- 在调用链的结尾清除调用链路类型标识符
---@param ev string
function F.careat_trace_type(ev)
	runtime.trace.type = ev
end

--- 清除临时调用链路类型标记
function F.clear_trace_type()
	runtime.trace.type = nil
end

--- 向日志队列中添加任务
---@param log LogMessage
function F.push_log(log)
	if runtime.stopped then
		return
	end
	local now = vim.uv.hrtime()
	local trace_type = runtime.trace.type or log.name .. log.source
	local cache = runtime.event_cache[trace_type]
	if cache == nil then
		runtime.event_cache[trace_type] = {
			trace_id = log.trace,
			logs = { log },
			push_time = now,
			already = false,
		}
	elseif not cache.already and cache.trace_id == log.trace then
		--- 相同调用链的一部分
		table.insert(cache.logs, log)
	else
		--- 同类型但调用链ID不同直接丢弃
		--- cache.already = true 舍弃所有内容 无论trace_id 是否相同
	end
end

--- 检查runtime.event_cache是否满足了写入条件
function F.log_tick()
	if runtime.stopped then
		return
	end
	local traces = {}
	for k, v in pairs(runtime.event_cache) do
		local now = vim.uv.hrtime()
		if not v.already and now - v.push_time >= Config.max_time then
			v.already = true
			table.insert(traces, v)
			runtime.event_cache[k] = nil
		end
	end
	local ok, err = F.write_log(traces)
	if not ok then
		runtime.stopped = true
		vim.notify(vim.inspect(err), F.error)
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

--- 按照日期写入日志文件
---@param traces TraceEvent[]
---@return boolean ok
---@return Error? err
function F.write_log(traces)
	local date = os.date("%Y-%m-%d", os.time())
	local path = string.format("%s/%s.log", Config.log_path, date)
	local function format_log(log)
		local inspect_opts = {
			depth = 4,
			indent = "    ",
		}
		local l
		if not log.level or log.level == F.info then
			l = "INFO"
		elseif log.level == F.warn then
			l = "WARN"
		elseif log.level == F.error then
			l = "ERROR"
		end

		local s = math.floor(log.time / 1000)
		local ms = log.time % 1000
		local time = os.date("%Y-%m-%d %H:%M:%S", s) .. "." .. string.format("%03d", ms)

		local header =
			string.format("[-- [%s] %s | %5s | %s | Trace: %d --]\n", time, log.source, l, log.name, log.trace or 0)
		local body = string.format(
			"context=%s\nerror=%s\ninfo=%s\n-----     End     -----\n",
			vim.inspect(log.context, inspect_opts) or "-----",
			vim.inspect(log.error, inspect_opts) or "-----",
			vim.inspect(log.info, inspect_opts) or "-----"
		)
		return header .. body
	end
	table.sort(traces, function(a, b)
		return a.push_time < b.push_time
	end)
	for _, v in ipairs(traces) do
		table.sort(v.logs, function(a, b)
			return a.time < b.time
		end)
		for _, log in ipairs(v.logs) do
			local ok, err = iolib.write(path, format_log(log))
			if not ok then
				return false, err
			end
		end
	end
	return true, nil
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

return F
