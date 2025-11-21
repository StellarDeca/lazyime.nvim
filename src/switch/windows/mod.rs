//! Windows 在跨进程的输入法控制下存在 IMM 与 TSF
//! IMM 可以跨进程调用，但是实现坑点很多，而且会被 TSF 逐步取代
//! TSF 虽然实现更加简单，但是不能跨进程工作
//! 最终还是使用win32 api 进行基于 IMM 与 Language 输入法控制。查询、控制输入法的内部状态存在困难
//!
//! 即使使用 win32 api， 跨进程 切换 输入法内部状态依旧不可行； 只能切换 键盘布局
//!

mod focus;
mod switch;

use super::InputMethodMode;
use focus::*;
use switch::*;
use windows::Win32::Foundation::HWND;

const NATIVE_LANGUAGE_ID: [u16; 1] = [2052];
const ENGLISH_LANGUAGE_ID: [u16; 1] = [1033];

pub(super) struct WinInputMethodController {
    /*
    native 与 english 为语言ID，值为0时表示不存在（不可用）
     */
    native: u16,
    english: u16,
    ground_handle: HWND,
}
impl WinInputMethodController {
    pub(super) fn new() -> Result<Self, String> {
        /*
        初始化数据结构体
        同时判断系统是否支持两种语言或者仅有一个语言支持内部状态切换
         */
        let languages = match get_available_languages() {
            Ok(languages) => languages,
            Err(e) => return Err(e),
        };
        let (native, english) = Self::check_supported_languages(&languages);

        if native == 0 && english == 0 {
            return Err("Windows Input Method Config is not available to control!".to_string());
        };

        // 尝试初始化焦点窗口
        let ground_handle = match get_ground_handle() {
            Ok(focus) => focus,
            Err(e) => return Err(e),
        };

        Ok(Self {
            native,
            english,
            ground_handle,
        })
    }

    pub(super) fn get_mode(&self) -> InputMethodMode {
        let active = get_active_language(&get_context_handle(&self.ground_handle));
        if active != self.native {
            InputMethodMode::English
        } else {
            InputMethodMode::Native
        }
    }

    pub(super) fn switch_mode(&self, target_mode: InputMethodMode) -> bool {
        match target_mode {
            InputMethodMode::Native => {
                set_active_language(&get_context_handle(&self.ground_handle), self.native)
            }
            InputMethodMode::English => {
                set_active_language(&get_context_handle(&self.ground_handle), self.english)
            }
        }
    }

    fn check_supported_languages(languages: &[u16]) -> (u16, u16) {
        // 按照 从左到右 匹配，匹配成功则返回
        let native_avail_id = NATIVE_LANGUAGE_ID.iter().find(|&id| languages.contains(id));
        let english_avail_id = ENGLISH_LANGUAGE_ID.iter().find(|&id| languages.contains(id));
        (*native_avail_id.unwrap_or(&0), *english_avail_id.unwrap_or(&0))
    }
}
