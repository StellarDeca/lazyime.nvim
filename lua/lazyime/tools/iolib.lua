----- 路径构建模块 -----

local F = {}

function F.root()
	return vim.fn.stdpath("state") .. "/lazyime"
end

function F.get_server_path()
	local static = F.root() .. "/server/"
	if vim.fn.has("win32") == 1 then
		return static .. "LazyInputSwitcher.exe"
	else
		return static .. "LazyInputSwitcher"
	end
end

function F.get_log_path()
	return F.root() .. "/logs"
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
	if not ok then
		local stat2 = vim.uv.fs_stat(path)
		if stat2 and stat2.type == "directory" then
			return
		end
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

--- 清理目录中超过指定天数的文件
---@param dir string 目录路径
---@param max_days integer 保留天数
---@param date_parser fun(name:string):osdate? 从文件名解析日期
---@return boolean ok
---@return Error? err
function F.gc_by_date(dir, max_days, date_parser)
	local stat = vim.uv.fs_stat(dir)
	if not stat or stat.type ~= "directory" then
		return false, { "FileClean", "Path exists but is not a directory: " .. dir, true, false }
	end

	local now = os.time()
	local req, err1 = vim.uv.fs_scandir(dir)
	if not req then
		return false, { "FileClean", tostring(err1), true, false }
	end

	while true do
		local name, typ = vim.uv.fs_scandir_next(req)
		if not name then
			break
		end

		if typ ~= "file" then
			goto continue
		end

		local date = date_parser(name)
		if not date then
			goto continue
		end

		local file_time = os.time(date)
		local age_days = math.floor((now - file_time) / 86400)

		if age_days >= max_days then
			local full = dir .. "/" .. name
			pcall(vim.uv.fs_unlink, full)
		end
		::continue::
	end
end

return F
