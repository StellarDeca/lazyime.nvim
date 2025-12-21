----- 路径构建模块 -----

local F = {}

function F.root()
	local path = vim.api.nvim_get_runtime_file("lua/lazyime/init.lua", false)[1]
	return vim.fn.fnamemodify(path, ":p:h")
end

function F.get_server_path()
	local static = F.root() .. "/static/"
	if vim.fn.has("win32") == 1 then
		return static .. "windows/LazyInputSwitcher.exe"
	elseif vim.fn.has("mac") == 1 then
		local info = vim.uv.os_uname()
		local arch_dir = ""
		if info.machine == "arm64" then
			arch_dir = "macos/arm64"
		else
			arch_dir = "macos/intel64"
		end
		return static .. arch_dir .. "LazyInputSwitcher"
	else
		return static .. "linux/LazyInputSwitcher"
	end
end

function F.get_log_path()
	local log = vim.fn.stdpath("state") .. "/lazyime/logs/"
end

local function ensure_dir(path)
	local stat = vim.uv.fs_stat(path)
	if stat then
		if stat.type ~= "directory" then
			error("Path exists but is not a directory: " .. path)
		end
		return
	end

	local parent = vim.fs.dirname(path)
	if parent and parent ~= path then
		ensure_dir(parent)
	end

	local ok, err = vim.uv.fs_mkdir(path, 493) -- 0755
	if not ok and err ~= "EEXIST" then
		error(err)
	end
end

--- 检查路径是否存在
--- 不存在则创建
---@return boolean ok
---@return Error? err
function F.make_dir(path)
	local ok, err = pcall(function()
		ensure_dir(path)
	end)
	if not ok then
		return false, { "MakeDirIo", tostring(err), true, false }
	end
	return true, nil
end

--- 检查指定文件是否存在
--- 不存在则创建
---@return boolean ok
---@return Error? err
function F.create_file(path)
	local ok, err = pcall(function()
		local stat = vim.uv.fs_stat(path)
		if stat then
			if stat.type ~= "file" then
				error("Path exists but is not a file: " .. path)
			end
			return
		end

		local dir = vim.fs.dirname(path)
		if dir then
			ensure_dir(dir)
		end

		local fd, open_err = vim.uv.fs_open(path, "a", 420) -- 0644
		if not fd then
			error(open_err)
		end
		vim.uv.fs_close(fd)
	end)

	if not ok then
		return false, { "CreateFileIo", tostring(err), true, false }
	end

	return true, nil
end

--- 向指定文件中写入内容
---@return boolean ok
---@return Error? err
function F.write(path, msg)
	local ok, err = pcall(function()
		local okf, ferr = F.create_file(path)
		if not okf and ferr then
			error(ferr.error)
		end

		local fd, open_err = vim.uv.fs_open(path, "a+", 420)
		if not fd then
			error(open_err)
		end

		vim.uv.fs_write(fd, msg)
		vim.uv.fs_close(fd)
	end)

	if not ok then
		return false, { "LogWriteIo", tostring(err), true, false }
	end

	return true, nil
end

return F
