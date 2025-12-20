----- 日志模块 -----

local F = {}

--- 错误样式
---@class Error
---@field name string 错误名
---@field error string 错误信息
---@field fatal boolean true-错误可恢复-false-不可恢复
---@field panic boolean true-程序应当崩溃

--- 返回错误
---@param name string
---@param error string
---@param fatal boolean
---@param panic boolean?
---@return Error
function F.make_error(name, error, fatal, panic)
	if panic == nil then
		panic = false
	end
	return {
		name = name,
		error = error,
		fatal = fatal,
		panic = panic,
	}
end

return F
