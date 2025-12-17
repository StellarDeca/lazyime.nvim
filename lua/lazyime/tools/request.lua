----- 与服务器端对应的 lua request 结构表 -----

local F = {}

---@class Cursor
---@field row integer        -- 0-based
---@field column integer     -- UTF-16 column, 0-based

---@alias CommandMode
---| '"Analyze"'
---| '"MethodOnly"'
---| '"Switch"'
---| '"Exit"'

---@class AnalyzeParams
---@class AnalyzeParams
---@field code string
---@field language string
---@field cursor Cursor

---@class MethodOnlyParams
---@field mode '"Native"' | '"English"'

---@class SwitchParams
---@field code string
---@field language string
---@field cursor Cursor

---@class AnalyzeRequest
---@field cid integer
---@field command '"Analyze"'
---@field params AnalyzeParams

---@class MethodOnlyRequest
---@field cid integer
---@field command '"MethodOnly"'
---@field params MethodOnlyParams

---@class SwitchRequest
---@field cid integer
---@field command '"Switch"'
---@field params SwitchParams

---@class ExitRequest
---@field cid integer
---@field command '"Exit"'
---@field params table  -- 空表 {}

---@alias ClientRequest
---| AnalyzeRequest
---| MethodOnlyRequest
---| SwitchRequest
---| ExitRequest

---@param cid integer
---@param code string
---@param language string
---@param cursor Cursor
---@return AnalyzeRequest
function F.creat_analysis_req(cid, code, language, cursor)
	return {
		cid = cid,
		command = "Analyze",
		params = {
			code = code,
			language = language,
			cursor = cursor,
		},
	}
end

---@param cid integer
---@param mode '"Native"' | '"English"'
---@return MethodOnlyRequest
function F.create_method_only_req(cid, mode)
	return {
		cid = cid,
		command = "MethodOnly",
		params = {
			mode = mode,
		},
	}
end

---@param cid integer
---@param code string
---@param language string
---@param cursor Cursor
---@return SwitchRequest
function F.create_switch_req(cid, code, language, cursor)
	return {
		cid = cid,
		command = "Switch",
		params = {
			code = code,
			language = language,
			cursor = cursor,
		},
	}
end

---@param cid integer
---@return ExitRequest
function F.create_exit_req(cid)
	return {
		cid = cid,
		command = "Exit",
		params = {},
	}
end

---@param req ClientRequest
---@return string
function F.to_json_message(req)
	return vim.json.encode(req)
end

return F
