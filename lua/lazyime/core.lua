----- 核心插件逻辑 -----

local F = {}
local network = require("lazyime.tools.network")
local pathlib = require("lazyime.tools.pathlib")
local request = require("lazyime.tools.request")

--- 启动并连接服务器
---@return uv.uv_tcp_t
function F.run_server()
	local path = pathlib.get_server_path()
	return network.start_server(path)
end

--- 切换到指定状态输入法
---@param runtime table
---@param mode InputMethodMode
---@return boolean
function F.switch(runtime, mode)
	local req = request.create_method_only_req(runtime.cid, mode)
	local res = network.request(runtime.tcp, req)
	if not res or not res.success then
		return false
	end
	return res.result.method == mode
end

--- 对buf进行语法分析
---@param runtime table
---@return GrammarMode
function F.grammer_analysis(runtime)
	local code, language = F.get_buffer()
	local cursor = F.get_cursor()

	local req = request.creat_analysis_req(runtime.cid, code, language, cursor)
	local res = network.request(runtime.tcp, req)

	if not res or not res.success then
		return runtime.grammar
	end
	return res.result.grammar
end

--- 对当前buf进行语法分析并切换到对应输入法
---@param runtime table
---@return GrammarMode grammar, InputMethodMode mode
function F.grammar_analysis_and_switch(runtime)
	local code, language = F.get_buffer()
	local cursor = F.get_cursor()

	local req = request.create_switch_req(runtime.cid, code, language, cursor)
	local res = network.request(runtime.tcp, req)

	if not res or not res.success then
		return runtime.grammar, runtime.method
	end
	return res.result.grammar, res.result.method
end

--- 获取当前 buffer 内容与类型
---@return string code, string language
function F.get_buffer()
	-- 获取文件内容与类型
	local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)
	local code = table.concat(lines, "\n")
	local language = vim.bo.filetype
	return code, language
end

--- 获取 当前 window 中 cursor 位置
---@return Cursor
function F.get_cursor()
	local pos = vim.api.nvim_win_get_cursor(0)
	return {
		row = pos[1] - 1,
		column = pos[2],
	}
end

return F
