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

#[derive(Debug)]
pub(super) enum InputMethodMode {
    Native,
    English,
}

pub(super) struct InputMethodStatus {
    #[cfg(target_os = "windows")]
    windows_controller: windows::WinInputMethodController,
}
impl InputMethodStatus {
    pub(super) fn new(pid: u32) -> Result<InputMethodStatus, String> {
        #[cfg(target_os = "windows")]
        let windows_controller = match windows::WinInputMethodController::new(pid) {
            Ok(windows_controller) => windows_controller,
            Err(err) => return Err(err),
        };
        Ok(InputMethodStatus { windows_controller })
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

pub fn test() {
    use std::time::Instant;
    let total_start = Instant::now();

    // 阶段1: windows::test()
    let stage1_start = Instant::now();
    windows::test();
    let stage1_duration = stage1_start.elapsed();
    println!("🔄 windows::test() 用时: {:?}", stage1_duration);

    // 阶段2: 创建 InputMethodStatus
    let stage2_start = Instant::now();
    let status = match InputMethodStatus::new(windows::get_pid()) {
        Ok(status) => status,
        Err(err) => panic!("{}", err),
    };
    let stage2_duration = stage2_start.elapsed();
    println!("🔄 InputMethodStatus::new() 用时: {:?}", stage2_duration);

    // 阶段3: 获取和切换模式
    let stage3_start = Instant::now();
    println!("{:?}", status.get_mode());
    status.switch_mode();
    println!("{:?}", status.get_mode());
    let stage3_duration = stage3_start.elapsed();
    println!("🔄 模式操作用时: {:?}", stage3_duration);

    let total_duration = total_start.elapsed();
    println!("✅ 总用时: {:?}", total_duration);
}
