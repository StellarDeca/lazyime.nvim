----- 核心插件逻辑 -----

local F = {}
local logger = require("lazyime.tools.log")
local iolib = require("lazyime.tools.iolib")
local network = require("lazyime.tools.network")
local request = require("lazyime.tools.request")
local response = require("lazyime.tools.response")

--- 发送请求并接收响应
---@param tcp uv.uv_tcp_t
---@param req ClientRequest
---@return ClientResponse? res, Error? err
function F.request(tcp, req)
	local msg, err1 = request.to_json_message(req)
	if not msg then
		return nil, err1
	end

	local ok, err2 = network.send_message(tcp, msg)
	if not ok then
		return nil, err2
	end

	local raw, err3 = network.recv_message(tcp)
	if not raw then
		return nil, err3
	end

	local res, err4 = response.from_json_message(raw)
	if not res then
		return nil, logger.make_error("ProtocolError", "failed to parse response", false, false)
	end

	return res, nil
end

--- 启动并连接服务器
---@return integer? port
---@return uv.uv_tcp_t? socket
---@return Error? error
function F.run_server()
	local path = iolib.get_server_path()
	return network.start_server(path)
end

--- 停止服务器实例
---@param runtime table
---@return true? ok, Error? error
function F.stop_server(runtime)
	local req = request.create_exit_req(runtime.cid)
	local msg, err = request.to_json_message(req)
	if not msg then
		return nil, err
	end
	return network.send_message(runtime.tcp, msg)
end

--- 获取cid
---@param runtime table
---@return integer, Error? reason
function F.get_cid(runtime)
	local req = request.create_switch_req(0, "", "Lua", F.get_cursor())
	local res, err = F.request(runtime.tcp, req)
	if not res or not res.success then
		return 0, err
	end
	return res.cid
end

--- 切换到指定状态输入法
---@param runtime table
---@param mode InputMethodMode
---@return boolean res
---@return Error? reason 失败原因
function F.switch(runtime, mode)
	local req = request.create_method_only_req(runtime.cid, mode)
	local res, err = F.request(runtime.tcp, req)
	if not res or not res.success then
		return false, err
	end
	return res.result.method == mode, nil
end

--- 对buf进行语法分析
---@param runtime table
---@return GrammarMode
---@return Error? reason 失败原因
function F.grammer_analysis(runtime)
	local code, language = F.get_buffer()
	local cursor = F.get_cursor()

	local req = request.creat_analysis_req(runtime.cid, code, language, cursor)
	local res, err = F.request(runtime.tcp, req)

	if not res or not res.success then
		return runtime.grammar, err
	end
	return res.result.grammar, nil
end

--- 对当前buf进行语法分析并切换到对应输入法
---@param runtime table
---@return GrammarMode grammar
---@return InputMethodMode mode
---@return Error? reason 错误原因
function F.grammar_analysis_and_switch(runtime)
	local code, language = F.get_buffer()
	local cursor = F.get_cursor()

	local req = request.create_switch_req(runtime.cid, code, language, cursor)
	local res, err = F.request(runtime.tcp, req)

	if not res or not res.success then
		return runtime.grammar, runtime.method, err
	end
	return res.result.grammar, res.result.method, nil
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
