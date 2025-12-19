------ lazyime 入口文件 ------

local F = {}
local tasks = require("lazyime.core.tasks")
local core = require("lazyime.core.core")
local time = require("lazyime.tools.time")
local AutoCmdsGroup = vim.api.nvim_create_augroup("LazyIme", { clear = true })
local runtime = {
	cid = 0, -- 客户端ID
	tcp = nil, -- tcp socket
	grammar = nil, -- 光标位置语法状态
	method = "English", -- 当前输入法
}

local function add_task(ev, task)
	tasks.push_task({
		event = ev.event,
		params = runtime,
		task = task,
	})
	tasks.wake_work()
end

local function init_runtime()
	if not runtime.tcp or runtime.cid == 0 then
		local network = require("lazyime.tools.network")
		local request = require("lazyime.tools.request")
		local tcp = core.run_server()
		local req = request.create_method_only_req(runtime.cid, "English")
		local res = network.request(tcp, req)
		if not res or not res.success then
			error("Failed to init runtime!")
		end
		runtime.tcp = tcp
		runtime.cid = res.cid
		runtime.grammar = nil
		runtime.method = res.result.method
	end
end

function F.setup(opts)
	opts = opts or {}

	--- VimEnter, VimLeave, BufEnter AutoCmd
	vim.api.nvim_create_autocmd("UIEnter", {
		group = AutoCmdsGroup,
		callback = function(ev)
			local task = function(params)
				init_runtime()
				time.sleep(200)
				local max_try = 3
				while max_try >= 0 do
					if core.switch(params, "English") then
						runtime.method = "English"
						break
					else
						runtime.method = "Native"
					end
				end
			end
			add_task(ev, task)
		end,
	})

    vim.api.nvim_create_autocmd("VimLeave", {
		group = AutoCmdsGroup,
		callback = function(ev)
			add_task(ev, core.stop_server)
		end,
	})
    --]]

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
			local task = function(param)
				local grammar, method = core.grammar_analysis_and_switch(param)
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
				core.switch(param, "English")
			end
			add_task(ev, task)
		end,
	})

	--- CursorMovedI TextChangedI Autocmd
	vim.api.nvim_create_autocmd({ "CursorMovedI", "TextChangedI" }, {
		group = AutoCmdsGroup,
		callback = function(ev)
			local task = function(param)
				--- 仅仅在 grammar 发生变化时进行切换
				local grammar = core.grammer_analysis(param)
				if param.grammar ~= grammar then
					local mode
					if grammar == "Code" then
						mode = "English"
					else
						mode = "Native"
					end
					local method = core.switch(param, mode)
					param.grammar = grammar
					param.method = method
				end
			end
			add_task(ev, task)
		end,
	})
end

return F
