------ lazyime 入口文件 ------

local F = {}
local logger = require("lazyime.tools.log")
local tasks = require("lazyime.core.tasks")
local core = require("lazyime.core.core")
local time = require("lazyime.tools.time")
local AutoCmdsGroup = vim.api.nvim_create_augroup("LazyIme", { clear = true })
local runtime = {
	cid = 0, -- 客户端ID
	port = 0, -- 服务端端口号
	tcp = nil, -- tcp socket
	grammar = nil, -- 光标位置语法状态
	method = "English", -- 当前输入法
}

--- 向 task queue 中添加任务
local function add_task(ev, task)
	tasks.push_task({
		event = ev.event,
		params = runtime,
		task = task,
	})
	tasks.wake_work()
end

--- 错误处理函数
---@param err Error?
local function handle_error(err)
	if not err then
		return nil
	end
	if err.panic then
		local ok, err_ = core.stop_server(runtime)
		local error = err.error
		if not ok and err_ then
			error = error .. "\n" .. err_.error
		end
		vim.notify(("Lazyime panic! Error: %s"):format(error), vim.log.levels.ERROR)
	else
		vim.notify(("Lazyime warn: %s"):format(err.error), vim.log.levels.WARN)
	end
	--- 记录错误
	vim.print("error handle", err)
end

--- 初始化 runtime 缓存表
--- 确保只初始化一次
local function init_runtime()
	if runtime.port == 0 or not runtime.tcp or runtime.cid == 0 then
		local port, socket, err = core.run_server()
		if not port or not socket then
			handle_error(err)
		end
		runtime.port = port
		runtime.tcp = socket

		local cid, err1 = core.get_cid(runtime)
		if cid == 0 then
			handle_error(err1)
		end
		runtime.cid = cid

		local mode = "English"
		local success, err2 = core.switch(runtime, "English")
		if success == nil then
			mode = "Native"
			handle_error(err2)
		end
		runtime.method = mode
		runtime.grammar = nil
	end
end

--- 排除非文件
--- 排除插件窗口
--- return true should ignore
---@param ev table
---@return boolean is_ignore
local function ignore_buffer(ev)
	-- 排除非文件 buffer
	local buftype = vim.api.nvim_get_option_value("buftype", { buf = ev.buf })
	if buftype ~= "" then
		return true
	end
	-- 排除插件窗口
	local ft = vim.api.nvim_get_option_value("filetype", { buf = ev.buf })
	local ignore_ft = { "TelescopePrompt", "NvimTree", "lazy", "mason", "notify" }
	if vim.tbl_contains(ignore_ft, ft) then
		return true
	end
	return false
end

function F.setup(opts)
	opts = opts or {}

	-- VimEnter, VimLeave, BufEnter AutoCmd
	vim.api.nvim_create_autocmd("FocusGained", {
		group = AutoCmdsGroup,
		callback = function(ev)
			local task = function(params)
				init_runtime()
				time.sleep(100)
				local ok, err = core.switch(params, "English")
				if ok then
					runtime.method = "English"
				else
					handle_error(err)
					runtime.method = "Native"
				end
			end
			add_task(ev, task)
		end,
	})

	vim.api.nvim_create_autocmd("VimLeave", {
		group = AutoCmdsGroup,
		callback = function(ev)
			local task = function(params)
				local ok, err = core.stop_server(params)
				if not ok then
					handle_error(err)
				end
			end
			add_task(ev, task)
		end,
	})

	vim.api.nvim_create_autocmd("BufEnter", {
		group = AutoCmdsGroup,
		callback = function(ev)
			add_task(ev, function(_)
				init_runtime()
			end)
		end,
	})

	--- InsertEnter InsertLeave AutoCmd
	vim.api.nvim_create_autocmd("InsertEnter", {
		group = AutoCmdsGroup,
		callback = function(ev)
			if ignore_buffer(ev) then
				return
			end
			local task = function(param)
				local grammar, method, err = core.grammar_analysis_and_switch(param)
				if err then
					handle_error(err)
				end
				param.grammar = grammar
				param.method = method
			end
			add_task(ev, task)
		end,
	})

	vim.api.nvim_create_autocmd("InsertLeave", {
		group = AutoCmdsGroup,
		callback = function(ev)
			local task = function(param)
				local ok, err = core.switch(param, "English")
				if not ok then
					handle_error(err)
				end
			end
			add_task(ev, task)
		end,
	})

	--- CursorMovedI TextChangedI Autocmd
	vim.api.nvim_create_autocmd({ "CursorMovedI", "TextChangedI" }, {
		group = AutoCmdsGroup,
		callback = function(ev)
			if ignore_buffer(ev) then
				return
			end
			local task = function(param)
				--- 仅仅在 grammar 发生变化时进行切换
				local grammar, err = core.grammer_analysis(param)
				if err then
					handle_error(err)
				elseif param.grammar ~= grammar then
					local mode
					if grammar == "Code" then
						mode = "English"
					else
						mode = "Native"
					end
					local method, err1 = core.switch(param, mode)
					if err1 then
						handle_error(err1)
					end
					param.grammar = grammar
					param.method = method
				end
			end
			add_task(ev, task)
		end,
	})
end

return F
