//! "Start with Windows" via the per-user Run key: no elevation, no extra crates.

use std::path::Path;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const VALUE: &str = "MyFlow";

#[cfg(windows)]
fn reg(args: &[&str]) -> Option<std::process::Output> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    std::process::Command::new("reg")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()
}

#[cfg(windows)]
pub fn apply(enabled: bool, exe: &Path) {
    if enabled {
        let cmd = format!("\"{}\"", exe.display());
        reg(&[
            "add", RUN_KEY, "/v", VALUE, "/t", "REG_SZ", "/d", &cmd, "/f",
        ]);
    } else {
        reg(&["delete", RUN_KEY, "/v", VALUE, "/f"]);
    }
}

#[cfg(not(windows))]
pub fn apply(_enabled: bool, _exe: &Path) {}
