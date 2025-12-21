----- 与服务器端对应的 lua reponse 结构表 -----

local F = {}

---@alias GrammarMode
---| '"Code"'
---| '"Comment"'

---@alias InputMethodMode
---| '"Native"'
---| '"English"'

---@class AnalyzeResult
---@field grammar GrammarMode

---@class MethodOnlyResult
---@field method InputMethodMode

---@class SwitchResult
---@field grammar GrammarMode
---@field method InputMethodMode

---@alias CommandResult
---| AnalyzeResult
---| MethodOnlyResult
---| SwitchResult

---@class SuccessResponse
---@field cid integer
---@field success true
---@field error nil
---@field result CommandResult

---@class ErrorResponse
---@field cid integer
---@field success false
---@field error string
---@field result nil

---@alias ClientResponse
---| SuccessResponse
---| ErrorResponse

---@param msg string
---@return ClientResponse? response, Error? error
function F.from_json_message(msg)
	local ok, result = pcall(vim.json.decode, msg)
	if not ok then
		return nil, {
			name = "JsonDecodeFailed",
			error = tostring(result),
			fatal = false,
			panic = true,
		}
	end
	return result, nil
end

return F
