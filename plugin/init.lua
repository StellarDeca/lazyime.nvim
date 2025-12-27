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

--- 下载指定仓库的最新版release文件
local function download_server()
	local dir = iolib.root() .. "/server/"
	local filename = environment()
	local url = "https://github.com/StellarDeca/LazyInputSwitcher/releases/latest/download/" .. filename
	local ok, err = iolib.make_dir(dir)
	if not ok and err then
		error(("Failed to create path:%s\n%s"):format(dir, err.error), vim.log.levels.ERROR)
	end
	if vim.fn.executable("curl") == 1 then
		vim.notify("Downloading LazyIME server...", vim.log.levels.INFO)
		-- 参数
		-- -f: HTTP 错误时不输出内容
		-- -L: 跟随GitHub Release重定向
		-- -o: 写入到指定文件路径
		-- --connect-timeout: 设置建立连接的最大时间
		local result = vim.fn.system({
			"curl",
			"-fLo",
			dir .. filename,
			"--connect-timeout",
			"30",
			url,
		})
		if vim.v.shell_error ~= 0 then
			error("Download failed! Error: " .. result)
		end
	else
		error("curl not found, could not download server!\n" .. url)
	end
end

--- 解压缩 server release文件
local function extra()
	local dir = iolib.root() .. "/server/"
	local path = dir .. environment()

	if path:match("%.zip$") then
		local ps_cmd =
			string.format("powershell -Command \"Expand-Archive -Path '%s' -DestinationPath '%s' -Force\"", path, dir)
		vim.fn.system(ps_cmd)
	elseif path:match("%.tar%.gz$") then
		vim.fn.system({ "tar", "-xzf", path, "-C", dir })
	else
		error("Unknown archive format: " .. path)
	end

	-- 修复执行权限 仅限linux、macos
	if vim.fn.has("win32") == 0 then
		local program = dir .. "/LazyInputSwitcher"
		if vim.loop.fs_stat(program) then
			vim.fn.system({ "chmod", "+x", program })
		end
	end
end

--- 初始化初始化layime server配置
--- 下载最新版本或指定版本server
vim.api.nvim_create_user_command("LazyimeInit", function()
	vim.notify("Start download Lazyime server", vim.log.levels.INFO)
	local ok, err = pcall(download_server)
	if not ok then
		vim.notify("Download failed: " .. err, vim.log.levels.ERROR)
		return
	end

	local ok2, err1 = pcall(extra)
	if not ok2 then
		vim.notify("Extract failed: " .. err1, vim.log.levels.ERROR)
		return
	end

	vim.notify("LazyIME server initialized successfully", vim.log.levels.INFO)
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

	vim.notify("Lazyime unload successful", vim.log.levels.INFO)
end, { desc = "LazyimeUnload" })
