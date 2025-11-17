/*
获取 获取焦点窗口的 句柄，并判断窗口是否失去焦点
 */
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

pub(super) fn get_context_handle(ground_handle: &HWND) -> HWND {
    /*
    获取
    当焦点窗口的 pid 与给定的 pid 相同时才认为是正确的焦点窗口。
     */
     unsafe { ImmGetDefaultIMEWnd(*ground_handle) }
}

pub(super) fn get_ground_handle() -> Result<HWND, String> {
    /*
    获取前台焦点窗口句柄
     */
    let handle = unsafe { GetForegroundWindow() };
    if handle.is_invalid() {
        Err("Failed to get foreground window".to_string())
    } else {
        Ok(handle)
    }
}
