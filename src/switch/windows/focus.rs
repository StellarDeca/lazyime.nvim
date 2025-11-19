//! 焦点窗口句柄 FFI 封装
//! 
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

/// 获取 隐藏的输入法窗口 句柄
pub(super) fn get_context_handle(ground_handle: &HWND) -> HWND {
    unsafe { ImmGetDefaultIMEWnd(*ground_handle) }
}

/// 获取前台焦点窗口句柄
pub(super) fn get_ground_handle() -> Result<HWND, String> {
    let handle = unsafe { GetForegroundWindow() };
    if handle.is_invalid() {
        Err("Failed to get foreground window".to_string())
    } else {
        Ok(handle)
    }
}
