/*
获取 获取焦点窗口的 句柄，并判断窗口是否失去焦点
 */
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub(super) struct FocusState {
    pub(super) target_handle: HWND,
    pub(super) is_focus: bool,
}
impl FocusState {
    pub(super) fn new(pid: u32) -> Result<FocusState, String> {
        /*
        当焦点窗口的 pid 与给定的 pid 相同时才认为是第一个焦点窗口。
         */
        let target_handle = match FocusState::get_focus_handle() {
            Ok(handle) => {
                let pid_: u32 = 0;
                unsafe { GetWindowThreadProcessId(handle, Some(pid_ as *mut u32)) };
                if pid != pid_ {
                    return Err(format!("Pid {} is not target process!", pid_));
                } else {
                    handle
                }
            }
            Err(err) => return Err(err),
        };

        Ok(FocusState {
            target_handle,
            is_focus: true,
        })
    }

    pub(super) fn get_focus_handle() -> Result<HWND, String> {
        let handle = unsafe { GetForegroundWindow() };
        if handle.is_invalid() {
            return Err("Failed to get foreground window".to_string());
        } else {
            Ok(handle)
        }
    }

    pub(super) fn is_focus(&self) -> bool {
        let handle = FocusState::get_focus_handle();
        match handle {
            Ok(now) => now == self.target_handle,
            Err(_) => false,
        }
    }

    pub(super) fn poll_focus(&mut self, wait_moment_ms: u64) {
        /*
        轮询检查 窗口是否失去焦点 或者 重新获得焦点
         */
        loop {
            let handle = match FocusState::get_focus_handle() {
                Ok(handle) => handle,
                Err(_) => {
                    if self.is_focus {
                        /*
                        首次失去焦点，设置失去焦点的状态并调用回调函数
                         */
                        self.is_focus = false;
                    };
                    continue;
                }
            };

            if handle != self.target_handle && self.is_focus {
                /*
                首次失去焦点，设置失去焦点的状态并调用回调函数
                 */
                self.is_focus = false;
            } else if handle == self.target_handle && !self.is_focus {
                /*
                首次恢复焦点，设置恢复焦点状态并调用回调函数
                 */
                self.is_focus = true;
            };

            sleep(Duration::from_millis(wait_moment_ms))
        }
    }
}
