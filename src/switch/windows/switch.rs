/*
根据 PID实现 输入法的内部状态查询与切换，不同语言之间的输入法切换

查询 输入法 内部状态
SendMessage(foregroundIME, WM_IME_CONTROL, IMC_GETCONVERSIONMODE, 0)

旧版微软拼音（win10）
    0 ->  English
    1 -> Chinese
新版微软拼音：
    0 -> English / Half shape
    1 -> Chinese / Half shape

    1024 -> English / Full shape
    1025 -> Chinese / Full shape

*/
use crate::core::InputMethodMode;
use std::fmt::Display;
use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL};
use windows::Win32::UI::TextServices::{
    CLSID_TF_InputProcessorProfiles, ITfInputProcessorProfiles, TF_LANGUAGEPROFILE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowThreadProcessId, SendMessageW, WM_IME_CONTROL, WM_INPUTLANGCHANGEREQUEST,
};

const IMC_GET_CONVERSION_MODE: usize = 0x001;
const IMC_SET_CONVERSION_MODE: usize = 0x002;

#[derive(Eq, PartialEq)]
pub(super) enum WinInputMethodMode {
    Native,
    English,
}
impl Display for WinInputMethodMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WinInputMethodMode::Native => write!(f, "Native"),
            WinInputMethodMode::English => write!(f, "English"),
        }
    }
}
impl WinInputMethodMode {
    pub(super) fn from_isize(value: isize) -> WinInputMethodMode {
        if value == 1 || value == 1025 {
            WinInputMethodMode::Native
        } else {
            WinInputMethodMode::English
        }
    }

    pub(super) fn to_isize(&self) -> isize {
        match self {
            WinInputMethodMode::English => 0,
            WinInputMethodMode::Native => 1,
        }
    }

    pub(super) fn to_method_mode(&self) -> InputMethodMode {
        match self {
            WinInputMethodMode::Native => InputMethodMode::Native,
            WinInputMethodMode::English => InputMethodMode::English,
        }
    }

    pub(super) fn from_method_mode(mode: &InputMethodMode) -> WinInputMethodMode {
        match mode {
            InputMethodMode::Native => WinInputMethodMode::Native,
            InputMethodMode::English => WinInputMethodMode::English,
        }
    }
}

pub(super) fn get_method_mode(focus_handle: &HWND) -> WinInputMethodMode {
    /*
    获取焦点窗口的 输入法隐藏窗口，向输入法隐藏窗口发送控制消息
     */
    let forge_handle = unsafe { ImmGetDefaultIMEWnd(*focus_handle) };
    unsafe {
        let res = SendMessageW(
            forge_handle,
            WM_IME_CONTROL,
            Some(WPARAM(IMC_GET_CONVERSION_MODE)),
            Some(LPARAM(0)), // 未使用的参数，设置为0
        );
        WinInputMethodMode::from_isize(res.0)
    }
}

pub(super) fn set_method_mode(focus_handle: &HWND, new_mode: WinInputMethodMode) -> bool {
    /*
    获取焦点窗口的 输入法隐藏窗口，向输入法隐藏窗口发送控制消息
     */
    let forge_handle = unsafe { ImmGetDefaultIMEWnd(*focus_handle) };
    unsafe {
        SendMessageW(
            forge_handle,
            WM_IME_CONTROL,
            Some(WPARAM(IMC_SET_CONVERSION_MODE)),
            Some(LPARAM(new_mode.to_isize())),
        );
    };
    new_mode == get_method_mode(focus_handle)
}

pub(super) fn get_available_languages() -> Result<Vec<u16>, String> {
    /*
    查询系统中可用的 language
     */
    let count = unsafe { GetKeyboardLayoutList(None) };
    if count <= 0 {
        Err(String::from("Not found available language!"))
    } else {
        let mut layouts: Vec<HKL> = Vec::with_capacity(count as usize);
        unsafe {
            layouts.set_len(count as usize);
        }
        unsafe {
            GetKeyboardLayoutList(Some(&mut layouts));
        }

        let mut res = Vec::with_capacity(layouts.len());
        for item in layouts {
            res.push((item.0 as u64 & 0xFFFF) as u16);
        }
        Ok(res)
    }
}

pub(super) fn get_active_language(focus_handle: &HWND) -> u16 {
    let tid = unsafe { GetWindowThreadProcessId(*focus_handle, None) };
    let keyboard_layout = unsafe { GetKeyboardLayout(tid) };
    (keyboard_layout.0 as u64 & 0xFFFF) as u16
}

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

pub(super) fn get_available_profiles(language: u16) -> Result<Vec<TF_LANGUAGEPROFILE>, String> {
    if !unsafe { CoInitializeEx(Some(std::ptr::null_mut()), COINIT_APARTMENTTHREADED).is_ok() } {
        return Err("Failed to init COM!".to_string());
    };

    let profile_interface: ITfInputProcessorProfiles = match unsafe {
        CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)
    } {
        Ok(ptr) => ptr,
        Err(err) => return Err(format!("Failed to init thread mgr!: {}", err)),
    };

    let enum_profiles = match unsafe { profile_interface.EnumLanguageProfiles(language) } {
        Ok(ep) => ep,
        Err(err) => {
            return Err(format!(
                "Fail to get profiles {}.\tLanguage ID is {}",
                err, language
            ));
        }
    };
    let mut profiles: Vec<TF_LANGUAGEPROFILE> = Vec::new();
    unsafe {
        loop {
            // 这里创建profile是为了分配内存供Next方法写入数据
            let mut profile = [TF_LANGUAGEPROFILE::default()];
            let mut fetched = 0u32;
            let ep = enum_profiles.Next(&mut profile, &mut fetched);
            if !ep.is_ok() || fetched == 0 {
                break;
            };
            profiles.push(profile[0]);
        }
    };
    Ok(profiles)
}
