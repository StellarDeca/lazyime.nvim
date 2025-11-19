
//! 输入法 FFI 封装
//! ```
//! SendMessage(foregroundIME, WM_IME_CONTROL, IMC_GET_CONVERSION_MODE, 0)
//! ```
//! 旧版微软拼音（win10）
//!
//!     0 ->  English
//!     1 -> Chinese
//! 新版微软拼音：
//!
//!     0 -> English / Half shape
//!     1 -> Chinese / Half shape
//!     1024 -> English / Full shape
//!     1025 -> Chinese / Full shape

use std::fmt::Display;
use crate::core::InputMethodMode;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::Ime::ImmGetDefaultIMEWnd;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL};
use windows::Win32::UI::TextServices::{CLSID_TF_InputProcessorProfiles, ITfInputProcessorProfiles, TF_LANGUAGEPROFILE, };
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, };
use windows::Win32::UI::WindowsAndMessaging::{GetWindowThreadProcessId, SendMessageW, WM_IME_CONTROL, WM_INPUTLANGCHANGEREQUEST, };

// Windows 输入法控制常量
/// 设定输入法转换模式（如中/英文输入）的控制消息 ID
const IMC_SET_CONVERSION_MODE: usize = 0x002;
/// 获取输入法转换模式的控制消息 ID
const IMC_GET_CONVERSION_MODE: usize = 0x001;

/// 表示 Windows 输入法当前处于的模式。
///
/// 这是一个平台特定的枚举，与通用 `InputMethodMode` 有映射关系
#[derive(Eq, PartialEq)]
pub(super) enum WinInputMethodMode {
    /// 开启非英文输入（例如：中文、日文输入）。
    Native,
    /// 开启英文输入模式。
    English,
}
// 格式化输出
impl Display for WinInputMethodMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WinInputMethodMode::Native => write!(f, "Native"),
            WinInputMethodMode::English => write!(f, "English"),
        }
    }
}
impl WinInputMethodMode {
    /// 将 Windows FFI 返回的 `isize` 值映射到 [`WinInputMethodMode`] 枚举。
    ///
    /// FFI 值 `1` 或 `1025` 通常表示非英文模式（Native）。
    pub(super) fn from_isize(value: isize) -> WinInputMethodMode {
        if value == 1 || value == 1025 {
            WinInputMethodMode::Native
        } else {
            WinInputMethodMode::English
        }
    }

    /// 将 [`WinInputMethodMode`] 转换为 FFI 消息所需的 `isize` 值。
    ///
    /// * `English` 对应 `0`。
    /// * `Native` 对应 `1`。
    pub(super) fn to_isize(&self) -> isize {
        match self {
            WinInputMethodMode::English => 0,
            WinInputMethodMode::Native => 1,
        }
    }

    /// 转换为 [`InputMethodMode`]
    pub(super) fn to_method_mode(&self) -> InputMethodMode {
        match self {
            WinInputMethodMode::Native => InputMethodMode::Native,
            WinInputMethodMode::English => InputMethodMode::English,
        }
    }

    /// 从 [`InputMethodMode`] 进行转换
    pub(super) fn from_method_mode(mode: &InputMethodMode) -> WinInputMethodMode {
        match mode {
            InputMethodMode::Native => WinInputMethodMode::Native,
            InputMethodMode::English => WinInputMethodMode::English,
        }
    }
}

/// 获取焦点窗口当前的输入法转换模式。
///
/// **工作原理:**
/// 1. 通过 [`ImmGetDefaultIMEWnd`] 获取输入法隐藏窗口句柄。
/// 2. 向该句柄发送 [`WM_IME_CONTROL`] 消息，携带 [`IMC_GET_CONVERSION_MODE`] 命令。
pub(super) fn get_method_mode(focus_handle: &HWND) -> WinInputMethodMode {
    // 获取焦点窗口的 输入法隐藏窗口，向输入法隐藏窗口发送控制消息
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

/// 设定焦点窗口的输入法转换模式。
///
/// **工作原理:**
/// 1. 获取输入法隐藏窗口句柄。
/// 2. 发送 [`WM_IME_CONTROL`] 消息，携带 [`IMC_SET_CONVERSION_MODE`] 和目标模式值。
///
/// **返回:** 模式设定是否成功（即设定后查询到的模式是否与目标模式一致）。
pub(super) fn set_method_mode(focus_handle: &HWND, new_mode: WinInputMethodMode) -> bool {
    // 获取焦点窗口的 输入法隐藏窗口，向输入法隐藏窗口发送控制消息
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

/// 查询给定语言 ID 下所有可用的输入处理器配置文件（Profile）。
///
/// **注意:** 此函数涉及 COM 初始化 (`CoInitializeEx`) 和 COM 对象的创建。
///
/// **返回:** 输入处理器配置文件列表 (`Vec<TF_LANGUAGEPROFILE>`)。
pub(super) fn get_available_profiles(language: u16) -> Result<Vec<TF_LANGUAGEPROFILE>, String> {
    // 初始化COM环境并获取TSF Profile 接口实例， 遍历Profile文件的迭代器
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
