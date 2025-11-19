/*
这个模块主要实现：
    其他语言 <==> 英文 输入法的切换
 */
   
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

use crate::core::InputMethodMode;

pub(super) struct Switcher {
    #[cfg(target_os = "windows")]
    windows_controller: windows::WinInputMethodController,
}
impl Switcher {
    pub(super) fn new() -> Result<Switcher, String> {
        #[cfg(target_os = "windows")]
        let windows_controller = match windows::WinInputMethodController::new() {
            Ok(windows_controller) => windows_controller,
            Err(err) => return Err(err),
        };
        Ok(Switcher { windows_controller })
    }

    pub(super) fn query(&self) -> InputMethodMode {
        #[cfg(target_os = "windows")]
        self.windows_controller.get_mode()
    }

    pub(super) fn switch(&self) -> bool {
        #[cfg(target_os = "windows")]
        match self.windows_controller.get_mode() {
            InputMethodMode::Native => self
                .windows_controller
                .switch_mode(InputMethodMode::English),
            InputMethodMode::English => {
                self.windows_controller.switch_mode(InputMethodMode::Native)
            }
        }
    }
}
