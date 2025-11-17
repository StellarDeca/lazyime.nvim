/*
这个模块主要实现：
    其他语言 <==> 英文 输入之间的切换 （可以是不同输入法之间的切换，也可以是通体输入法内部输入模式之间的切换）

    在同一输入法中控制输入法的输入：
        获取候选输入（如拼音）、候选框内容；
        设置输入法的输入模式（如 全角，半角；中/英输入；中/英标点输入 等）

    在当前的应用程序失去焦点后记忆当前的输入法状态并在再次成为焦点后对输入法状态进行恢复
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

    pub(super) fn get_mode(&self) -> InputMethodMode {
        #[cfg(target_os = "windows")]
        self.windows_controller.get_mode()
    }

    pub(super) fn switch_mode(&self) -> bool {
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
