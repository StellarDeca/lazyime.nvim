----- 网络通信模块 -----

local F = {}
local buffer = ""
local logger = require("lazyime.tools.log")

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
--- @return integer? port, uv.uv_tcp_t? socket, Error? err
function F.start_server(path)
	local co = coroutine.running()
	local buf = ""
	local port_found = false
	if not co then
		return nil,
			nil,
			logger.make_error("InvalidContext", "start_server must be called inside coroutine", false, true)
	end
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
				vim.schedule(function()
					coroutine.resume(co, port)
				end)
			end
		end,
	}, function(res)
		if res.code ~= 0 then
			vim.schedule(function()
				coroutine.resume(
					co,
					nil,
					logger.make_error("ServerExit", ("server exited with code %d"):format(res.code), false, true)
				)
			end)
		else
			--- 正常结束服务端,插件正常结束
		end
	end)
	local port, err = coroutine.yield()
	if not port then
		return nil, nil, err
	end
	local tcp = vim.uv.new_tcp()
	if not tcp then
		return port, nil, logger.make_error("TcpInitFailed", "failed to create tcp handle", false, true)
	end
	tcp:connect("127.0.0.1", port, function(connect_err)
		if connect_err then
			vim.schedule(function()
				coroutine.resume(co, false, logger.make_error("TcpConnectFailed", tostring(connect_err), false, true))
			end)
		else
			vim.schedule(function()
				coroutine.resume(co, true, nil)
			end)
		end
	end)
	local ok, err_ = coroutine.yield()
	if not ok then
		return port, nil, err_
	else
		return port, tcp, nil
	end
end

--- 发送消息
--- @param server uv.uv_tcp_t
--- @param msg string
--- @return true? result, Error? error
function F.send_message(server, msg)
	local co = coroutine.running()
	if not co then
		return nil, logger.make_error("InvalidContext", "send_message must be called inside coroutine", false, true)
	end
	local header = pack_u64be(#msg)
	local message = header .. msg
	server:write(message, function(err)
		if err then
			vim.schedule(function()
				coroutine.resume(co, false, logger.make_error("WriteFailed", tostring(err), true, false))
			end)
		else
			vim.schedule(function()
				coroutine.resume(co, true, nil)
			end)
		end
	end)
	return coroutine.yield()
end

--- 接收消息
---@param server uv.uv_tcp_t
---@return string? message, Error? err
function F.recv_message(server)
	local size
	local co = coroutine.running()
	if not co then
		return nil, logger.make_error("InvalidContext", "recv_message must be called inside coroutine", false, true)
	end
	server:read_start(function(err, chunk)
		--- 先读取 8 字节 头部
		--- 再读取消息
		if err then
			server:read_stop()
			vim.schedule(function()
				coroutine.resume(co, nil, logger.make_error("ReadFailed", tostring(err), false, false))
			end)
			return
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
					vim.schedule(function()
						coroutine.resume(co, message, nil)
					end)
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

return F
