----- 网络通信模块 -----

local F = {}
local request = require("lazyime.tools.request")
local response = require("lazyime.tools.response")

--- 返回 server socket套接字
--- @param path string
--- @return uv.uv_tcp_t
function F.start_server(path)
	local co = coroutine.running()
	local buf = ""
	vim.system({ path }, {
		stdout = function(_, data)
			if data then
				buf = buf .. data
			end
			local port = tonumber(buf:match("(%d+)\n"))
			coroutine.resume(co, port) --- 唤醒协程并返回system结果
		end,
	}, function(res)
		if res.code ~= 0 then
			error(("Server failed to start or exited with error code %d! Stderr: %s"):format(res.code, res.stderr))
		else
			--- 正常结束服务端,插件正常结束
		end
	end)

	--- 暂停协程 得到协程结果
	local port = coroutine.yield()
	local tcp, err_msg, err_name = vim.uv.new_tcp()
	if not tcp or not port then
		error(("Failed to create new TCP handle on port %s: %s (%s)."):format(tostring(port), err_msg, err_name))
	end

	tcp:connect("127.0.0.1", port, function(err)
		if err then
			error(("Failed to connect server! %s"):format(err))
		else
			coroutine.resume(co)
		end
	end)
	coroutine.yield()

	return tcp
end

--- 发送消息
--- @param server uv.uv_tcp_t
--- @param msg string
--- @return boolean result
function F.send_message(server, msg)
	local co = coroutine.running()
	server:send_buffer_size(512 * 1024)
	server:write(msg, function(err)
		if err then
			coroutine.resume(co, false)
		else
			coroutine.resume(co, true)
		end
	end)
	return coroutine.yield()
end

--- 接受消息
--- @param server uv.uv_tcp_t
--- @return string? message
function F.recv_message(server)
	local co = coroutine.running()
	server:read_start(function(err, chunk)
		server:read_stop()
		if err then
			coroutine.resume(co, nil)
		else
			coroutine.resume(co, chunk)
		end
	end)
	return coroutine.yield()
end

--- 发送请求并接受响应
--- @param tcp uv.uv_tcp_t
--- @param req ClientRequest
--- @return ClientResponse? res
function F.request(tcp, req)
	local msg = request.to_json_message(req)

	if not F.send_message(tcp, msg) then
		return nil
	end

	local raw = F.recv_message(tcp)
	if not raw then
		return nil
	end

	return response.from_json_message(raw)
end

return F
