//! 输入法 FFI 封装

use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, SendMessageW, WM_INPUTLANGCHANGEREQUEST};

/// 查询系统中可用的 language
pub(super) fn get_available_languages() -> Result<Vec<u16>, String> {
    // 查询系统中可用的 language
    let count = unsafe { GetKeyboardLayoutList(None) };
    if count <= 0 {
        Err(String::from("Not found available language!"))
    } else {
        let mut layouts: Vec<HKL> = Vec::with_capacity(count as usize);
        unsafe {
            layouts.set_len(count as usize);
            GetKeyboardLayoutList(Some(&mut layouts));
        }
        let mut res = Vec::with_capacity(layouts.len());
        for item in layouts {
            res.push((item.0 as u64 & 0xFFFF) as u16);
        }
        Ok(res)
    }
}

/// 获取当前活动的 语言ID
pub(super) fn get_active_language(focus_handle: &HWND) -> u16 {
    let tid = unsafe { GetWindowThreadProcessId(*focus_handle, None) };
    let keyboard_layout = unsafe { GetKeyboardLayout(tid) };
    (keyboard_layout.0 as u64 & 0xFFFF) as u16
}

/// 设置 活动语言为指定的 语言ID
///
/// 此时键盘布局会切换为指定的 语言ID的键盘布局
pub(super) fn set_active_language(focus_handle: &HWND, target_language: u16) -> bool {
    unsafe {
        SendMessageW(
            *focus_handle,
            WM_INPUTLANGCHANGEREQUEST,
            Some(WPARAM(0)), // 该参数不被使用，设置为0
            Some(LPARAM(target_language as isize)),
        )
    };
    target_language == get_active_language(focus_handle)
}
