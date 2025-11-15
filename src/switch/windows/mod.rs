/*
Windows 在跨进程的输入法控制下存在 IMM 与 TSF
    IMM 可以跨进程调用，但是实现坑点很多，而且会被 TSF 逐步取代
    TSF 虽然实现更加简单，但是不能跨进程工作

最终还是使用win32 api 进行基于 iMM 与 Language 输入法控制。查询、控制输入法的内部状态存在困难
 */

mod focus;
mod switch;

use crate::switch::InputMethodMode;
use focus::*;
use switch::*;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::TextServices::TF_LANGUAGEPROFILE;

const SUPPORT_PROFILES_GUID: [windows::core::GUID; 1] = [windows::core::GUID::from_u128(
    0xFA550B04_5AD7_411F_A5AC_CA038EC515D7,
)];
const NATIVE_LANGUAGE_ID: [u16; 1] = [2052];
const ENGLISH_LANGUAGE_ID: [u16; 1] = [1033];

pub(super) struct WinInputMethodController {
    /*
    native 与 english 为语言ID，值为0时表示不存在（不可用）
     */
    native: u16,
    english: u16,
    profile: Option<TF_LANGUAGEPROFILE>,
    ground_handle: HWND,
}
impl WinInputMethodController {
    pub(super) fn new(pid: u32) -> Result<Self, String> {
        /*
        初始化数据结构体
        同时判断系统是否支持两种语言或者仅有一个语言支持内部状态切换
         */
        let languages = match get_available_languages() {
            Ok(languages) => languages,
            Err(e) => return Err(e),
        };
        let (native, english) = Self::check_supported_languages(&languages);

        // 尝试进行输入法内部切换
        let profile = match Self::find_support_profile(native) {
            Ok(profile) => profile,
            Err(e) => return Err(e),
        };
        if native == 0 || (native != 0 && english == 0 && profile.is_none()) {
            return Err("Windows Input Method Config is not available to control!".to_string());
        };

        // 尝试初始化焦点窗口
        let ground_handle = match get_ground_handle(pid) {
            Ok(focus) => focus,
            Err(e) => return Err(e),
        };

        Ok(Self {
            native,
            english,
            profile,
            ground_handle,
        })
    }

    pub(super) fn get_mode(&self) -> InputMethodMode {
        if self.profile.is_none() {
            let active = get_active_language(&get_context_handle(&self.ground_handle));
            if active != self.native {
                InputMethodMode::English
            } else {
                InputMethodMode::Native
            }
        } else {
            // 这里还需要确保活动语言是Native
            if self.native != get_active_language(&get_context_handle(&self.ground_handle)) {
                return InputMethodMode::English;
            };
            get_method_mode(&get_context_handle(&self.ground_handle)).to_method_mode()
        }
    }

    pub(super) fn switch_mode(&self, target_mode: InputMethodMode) -> bool {
        if self.profile.is_none() {
            match target_mode {
                InputMethodMode::Native => {
                    set_active_language(&get_context_handle(&self.ground_handle), self.native)
                }
                InputMethodMode::English => {
                    set_active_language(&get_context_handle(&self.ground_handle), self.english)
                }
            }
        } else {
            // 这里还需要确保活动语言是Native
            if self.native != get_active_language(&get_context_handle(&self.ground_handle)) {
                if !set_active_language(&get_context_handle(&self.ground_handle), self.native) {
                    return false;
                };
            };
            set_method_mode(
                &get_context_handle(&self.ground_handle),
                WinInputMethodMode::from_method_mode(&target_mode),
            )
        }
    }

    fn check_supported_languages(languages: &[u16]) -> (u16, u16) {
        let native = NATIVE_LANGUAGE_ID[0];
        let english = ENGLISH_LANGUAGE_ID[0];

        let native_avail = if languages.contains(&native) {
            native
        } else {
            0
        };
        let english_avail = if languages.contains(&english) {
            english
        } else {
            0
        };

        (native_avail, english_avail)
    }

    fn find_support_profile(native: u16) -> Result<Option<TF_LANGUAGEPROFILE>, String> {
        if native == 0 {
            return Ok(None); // Native 语言不可用，无需查找输入法配置
        }

        // 尝试获取 Native 语言下的所有配置文件
        let profiles = match get_available_profiles(native) {
            Ok(p) => p,
            Err(e) => {
                return Err(e);
            }
        };

        // 查找我们支持的输入法 (如微软拼音)
        let support_profile = profiles
            .into_iter()
            .find(|profile| profile.guidProfile == SUPPORT_PROFILES_GUID[0]);

        Ok(support_profile)
    }
}

pub fn test() {
    let res = get_available_profiles(0x0804);
    // println!("{:?}", res);
    match res {
        Ok(profiles) => {
            for profile in profiles {
                if profile.guidProfile == SUPPORT_PROFILES_GUID[0] {
                    println!("Foreground");
                }
            }
        }
        Err(_) => {}
    }
}
