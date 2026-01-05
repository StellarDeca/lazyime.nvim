--- lazyime 用户命令
local iolib = require("lazyime.tools.iolib")

--- 判断操作系统环境并构建release文件名
---@return string
local function environment()
	if vim.fn.has("win32") == 1 then
		return "windows-x86_64.zip"
	elseif vim.fn.has("mac") == 1 then
		local arch = vim.loop.os_uname().machine
		if arch == "arm64" or arch == "aarch64" then
			return "macos-arm64.tar.gz"
		else
			return "macos-intel-x86_64.tar.gz"
		end
	else
		return "linux-x86_64.tar.gz"
	end
end

local function notify(msg, level)
	vim.schedule(function()
		vim.notify(msg, level)
	end)
end

--- 初始化初始化layime server配置
--- 下载最新版本或指定版本server
vim.api.nvim_create_user_command("LazyimeInit", function()
	local dir = iolib.root() .. "/server/"
	local filename = environment()
	local path = dir .. filename
	local url = "https://github.com/StellarDeca/LazyInputSwitcher/releases/latest/download/" .. filename
	local ok, err = iolib.make_dir(dir)
	if not ok and err then
		notify(("Failed to create path: %s\n%s"):format(dir, err.error), vim.log.levels.ERROR)
		return
	end

	if vim.fn.executable("curl") == 1 then
		notify("Start download LazyIME server...", vim.log.levels.INFO)
		-- 参数
		-- -f: HTTP 错误时不输出内容
		-- -L: 跟随GitHub Release重定向
		-- -o: 写入到指定文件路径
		-- --connect-timeout: 设置建立连接的最大时间
		vim.system({
			"curl",
			"-fLo",
			path,
			"--connect-timeout",
			"30",
			url,
		}, {}, function(res)
			notify("Download finished.\nExtracting server ..", vim.log.levels.INFO)
			if res.code ~= 0 then
				notify(("Failed to download server! Exit code: %d"):format(res.code), vim.log.levels.ERROR)
				return
			end
			-- 解压缩server
			local extract_cmd
			if path:match("%.zip$") then
				extract_cmd = {
					"powershell",
					"-Command",
					string.format("Expand-Archive -Path '%s' -DestinationPath '%s' -Force", path, dir),
				}
			elseif path:match("%.tar%.gz$") then
				extract_cmd = { "tar", "-xzf", path, "-C", dir }
			else
				notify("Unknown archive format: " .. path, vim.log.levels.ERROR)
				return
			end
			vim.system(extract_cmd, {}, function(res_)
				if res_.code ~= 0 then
					notify(("Failed to eatract server! Exit code: %d"):format(res.code), vim.log.levels.ERROR)
				else
					notify("Sunccess install server! please reboot nvim", vim.log.levels.INFO)
				end
			end)
		end)
	else
		notify("curl not found, could not download server!\n" .. url, vim.log.levels.ERROR)
	end
end, { desc = "LazyimeInit" })

--- 下载最新版本的server版本
vim.api.nvim_create_user_command("LazyimeUpdate", function()
	vim.fn.delete(iolib.root() .. "/server", "rf")
	vim.cmd("LazyimeInit")
end, { desc = "LazuimeUpdateServer" })

--- 卸载lzyime
vim.api.nvim_create_user_command("LazyimeUnload", function()
	-- 删除自动命令
	vim.api.nvim_clear_autocmds({ group = "LazyIme" })

	-- 删除用户命令
	vim.api.nvim_del_user_command("LazyimeUnload")
	vim.api.nvim_del_user_command("LazyimeInit")
	vim.api.nvim_del_user_command("LazyimeUpdate")

	-- 卸载模块文件
	for k in pairs(package.loaded) do
		if k:match("^lazyime") then
			package.loaded[k] = nil
		end
	end

	notify("Lazyime unload successful", vim.log.levels.INFO)
end, { desc = "LazyimeUnload" })
