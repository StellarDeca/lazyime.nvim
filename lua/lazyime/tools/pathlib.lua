----- 路径构建模块 -----

local F = {}

function F.root()
	local path = vim.api.nvim_get_runtime_file("lua/lazyime/init.lua", false)[1]
	return vim.fn.fnamemodify(path, ":p:h")
end

function F.get_server_path()
	local static = F.root() .. "\\static"
	if vim.fn.has("win32") == 1 then
		return static .. "\\windows\\LazyInputSwitcher.exe"
	elseif vim.fn.has("mac") == 1 then
		local info = vim.uv.os_uname()
		local arch_dir = ""
		if info.machine == "arm64" then
			arch_dir = "\\macos\\arm64"
		else
			arch_dir = "\\macos\\intel64"
		end
		return static .. arch_dir .. "\\LazyInputSwitcher"
	else
		return static .. "\\linux\\LazyInputSwitcher"
	end
end

return F
