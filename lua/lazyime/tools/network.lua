----- 网络通信模块 -----

local F = {}
local buffer = ""
local request = require("lazyime.tools.request")
local response = require("lazyime.tools.response")

-- 发送时：将长度转为 8 字节大端序
local function pack_u64be(n)
	local low = n % 2 ^ 32
	local high = math.floor(n / 2 ^ 32)
	return string.char(
		math.floor(high / 2 ^ 24) % 256,
		math.floor(high / 2 ^ 16) % 256,
		math.floor(high / 2 ^ 8) % 256,
		high % 256,
		math.floor(low / 2 ^ 24) % 256,
		math.floor(low / 2 ^ 16) % 256,
		math.floor(low / 2 ^ 8) % 256,
		low % 256
	)
end

-- 接收时：将 8 字节大端序转回数字
local function unpack_u64be(s)
	local b1, b2, b3, b4, b5, b6, b7, b8 = string.byte(s, 1, 8)
	local high = b1 * 2 ^ 24 + b2 * 2 ^ 16 + b3 * 2 ^ 8 + b4
	local low = b5 * 2 ^ 24 + b6 * 2 ^ 16 + b7 * 2 ^ 8 + b8
	return high * 2 ^ 32 + low
end

--- 返回 server socket套接字
--- @param path string
--- @return uv.uv_tcp_t
function F.start_server(path)
	local co = coroutine.running()
	local buf = ""
	local port_found = false
	vim.system({ path }, {
		stdout = function(_, data)
			if data then
				buf = buf .. data
			end
			local port = tonumber(buf:match("(%d+)\n"))
			if port and not port_found then
				--- 确保回调只执行一次,只关心第一次输出的端口号
				--- 唤醒协程并返回system结果
				port_found = true
				coroutine.resume(co, port)
			end
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
	local header = pack_u64be(#msg)
	local message = header .. msg
	server:write(message, function(err)
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
	local size

	server:read_start(function(err, chunk)
		--- 先读取 8 字节 头部
		--- 再读取消息
		if err then
			coroutine.resume(co, nil)
		end

		buffer = buffer .. chunk
		while true do
			if not size then
				if #buffer >= 8 then
					size = tonumber(unpack_u64be(buffer:sub(1, 8)))
					buffer = buffer:sub(9)
				else
					-- 继续读取头部
					break
				end
			else
				if #buffer >= size then
					local message = buffer:sub(1, size)
					buffer = buffer:sub(size + 1)

					server:read_stop()
					coroutine.resume(co, message)
					break
				else
					-- 继续读取消息
					break
				end
			end
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
