use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use tauri::AppHandle;

use super::{HostPower, PlatformError, WakeOutcome, WakeTarget};

pub(super) fn initialize(app: &AppHandle) {
    crate::banner::bind_app(app.clone());
    crate::banner::install();
}

pub(super) fn prepare_launch() {
    crate::app_bundle::adopt_bundle();
}

pub(super) fn computer_name() -> String {
    if let Ok(output) = Command::new("scutil")
        .args(["--get", "ComputerName"])
        .output()
    {
        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }
    hostname::get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Mac".to_string())
}

pub(super) fn host_power() -> HostPower {
    crate::power::read()
}

pub(super) fn lock_session() -> Result<(), PlatformError> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to key code 12 using {control down, command down}")
        .output()
        .map_err(|err| {
            PlatformError::Unavailable(format!("Could not start macOS screen lock: {err}"))
        })?;
    if output.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(PlatformError::PermissionDenied(format!(
            "Allow Smart Connect to control System Events in Privacy & Security. {detail}"
        )))
    }
}

pub(super) fn open_path(path: &Path) -> Result<(), PlatformError> {
    let status = Command::new("open")
        .arg(path)
        .status()
        .map_err(|err| PlatformError::Unavailable(format!("Could not open the file: {err}")))?;
    if status.success() {
        Ok(())
    } else {
        Err(PlatformError::Failed("Could not open the file.".into()))
    }
}

pub(super) fn read_clipboard_text() -> Result<String, PlatformError> {
    let output = Command::new("pbpaste").output().map_err(|err| {
        PlatformError::Unavailable(format!("Could not read the Mac clipboard: {err}"))
    })?;
    if !output.status.success() {
        return Err(PlatformError::Failed(
            "Could not read the Mac clipboard.".into(),
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| PlatformError::Failed(format!("Clipboard text is not UTF-8: {err}")))
}

pub(super) fn write_clipboard_text(text: &str) -> Result<(), PlatformError> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| {
            PlatformError::Unavailable(format!("Could not write the Mac clipboard: {err}"))
        })?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).map_err(|err| {
            PlatformError::Failed(format!("Could not write the Mac clipboard: {err}"))
        })?;
    }
    let status = child.wait().map_err(|err| {
        PlatformError::Failed(format!("Could not finish writing the Mac clipboard: {err}"))
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(PlatformError::Failed(
            "Could not write the Mac clipboard.".into(),
        ))
    }
}

pub(super) fn show_banner(
    app_handle: &AppHandle,
    app: String,
    title: String,
    body: String,
    key: String,
    pkg: String,
    icon: Option<Vec<u8>>,
    on_unavailable: impl FnOnce() + Send + 'static,
) {
    crate::banner::bind_app(app_handle.clone());
    crate::banner::show(app, title, body, key, pkg, icon, on_unavailable);
}

pub(super) async fn push_wake_invites(
    targets: Vec<WakeTarget>,
) -> Result<WakeOutcome, PlatformError> {
    crate::wake::push_invites(targets)
        .await
        .map_err(PlatformError::Failed)
}

pub(super) fn ensure_camera_publisher() {
    crate::cam_publish::ensure();
}

pub(super) fn set_camera_published(on: bool) -> Result<(), PlatformError> {
    crate::cam_publish::set_open(on);
    Ok(())
}
