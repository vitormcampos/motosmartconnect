use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use super::{HostPower, PlatformError, WakeOutcome, WakeTarget};

pub(super) fn initialize() {}

pub(super) fn computer_name() -> String {
    hostname::get()
        .ok()
        .and_then(|value| value.into_string().ok())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Linux PC".into())
}

pub(super) fn host_power() -> HostPower {
    let Ok(entries) = std::fs::read_dir("/sys/class/power_supply") else {
        return HostPower::on_power();
    };

    let mut battery = None;
    let mut external_power = false;
    for entry in entries.flatten() {
        let path = entry.path();
        let kind = read_trimmed(&path.join("type")).unwrap_or_default();
        match kind.as_str() {
            "Battery" => {
                let capacity = read_trimmed(&path.join("capacity"))
                    .and_then(|value| value.parse::<i64>().ok())
                    .unwrap_or(100)
                    .clamp(0, 100);
                let status = read_trimmed(&path.join("status")).unwrap_or_default();
                let charging = status == "Charging";
                let plugged = external_power || charging || status == "Full";
                battery = Some(HostPower {
                    battery: capacity,
                    charging,
                    plugged,
                    battery_available: true,
                });
            }
            "Mains" | "USB" | "USB_C" | "USB_PD" => {
                external_power |= read_trimmed(&path.join("online")).as_deref() == Some("1");
            }
            _ => {}
        }
    }

    battery
        .map(|mut state| {
            state.plugged |= external_power;
            state
        })
        .unwrap_or_else(HostPower::on_power)
}

fn read_trimmed(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
}

pub(super) fn lock_session() -> Result<(), PlatformError> {
    let output = Command::new("loginctl")
        .arg("lock-session")
        .output()
        .map_err(|err| {
            PlatformError::Unavailable(format!(
                "Could not request a session lock through logind: {err}"
            ))
        })?;
    if output.status.success() {
        Ok(())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(PlatformError::Unsupported(format!(
            "The current desktop session did not accept a lock request. {detail}"
        )))
    }
}

pub(super) fn open_path(path: &Path) -> Result<(), PlatformError> {
    Command::new("xdg-open")
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|err| {
            PlatformError::Unavailable(format!("Could not open the file with the desktop: {err}"))
        })
}

pub(super) fn read_clipboard_text() -> Result<String, PlatformError> {
    if !is_wayland_session() {
        return Err(PlatformError::Unsupported(
            "Clipboard access is currently implemented for Wayland sessions only.".into(),
        ));
    }
    let output = Command::new("wl-paste")
        .arg("--no-newline")
        .output()
        .map_err(|err| {
            PlatformError::Unavailable(format!(
                "Install wl-clipboard to read the Wayland clipboard: {err}"
            ))
        })?;
    if !output.status.success() {
        return Err(PlatformError::Failed(
            "Could not read the Wayland clipboard.".into(),
        ));
    }
    String::from_utf8(output.stdout)
        .map_err(|err| PlatformError::Failed(format!("Clipboard text is not UTF-8: {err}")))
}

pub(super) fn write_clipboard_text(text: &str) -> Result<(), PlatformError> {
    if !is_wayland_session() {
        return Err(PlatformError::Unsupported(
            "Clipboard access is currently implemented for Wayland sessions only.".into(),
        ));
    }
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| {
            PlatformError::Unavailable(format!(
                "Install wl-clipboard to write the Wayland clipboard: {err}"
            ))
        })?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes()).map_err(|err| {
            PlatformError::Failed(format!("Could not write the Wayland clipboard: {err}"))
        })?;
    }
    let status = child.wait().map_err(|err| {
        PlatformError::Failed(format!(
            "Could not finish writing the Wayland clipboard: {err}"
        ))
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(PlatformError::Failed(
            "Could not write the Wayland clipboard.".into(),
        ))
    }
}

fn is_wayland_session() -> bool {
    let has_wayland_display = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    session_is_wayland(has_wayland_display, session_type.as_deref())
}

fn session_is_wayland(has_wayland_display: bool, session_type: Option<&str>) -> bool {
    has_wayland_display || session_type.is_some_and(|kind| kind.eq_ignore_ascii_case("wayland"))
}

pub(super) fn show_banner(
    _app: String,
    _title: String,
    _body: String,
    _key: String,
    _pkg: String,
    _icon: Option<Vec<u8>>,
    on_unavailable: impl FnOnce() + Send + 'static,
) {
    // Keep in-app notifications as the source of truth until a desktop backend is installed.
    on_unavailable();
}

pub(super) async fn push_wake_invites(
    targets: Vec<WakeTarget>,
) -> Result<WakeOutcome, PlatformError> {
    if targets.is_empty() {
        return Ok(WakeOutcome::default());
    }
    Err(PlatformError::Unsupported(
        "Bluetooth wake via BlueZ is not available yet. Open Smart Connect on the phone or reconnect over the local network.".into(),
    ))
}

pub(super) fn ensure_camera_publisher() {}

pub(super) fn set_camera_published(on: bool) -> Result<(), PlatformError> {
    if on {
        Err(PlatformError::Unsupported(
            "Virtual-camera publishing is not available in this Linux build yet.".into(),
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_desktop_without_a_battery_has_an_explicit_power_fallback() {
        let power = HostPower::on_power();
        assert!(!power.battery_available);
        assert!(power.plugged);
    }

    #[test]
    fn clipboard_is_available_only_for_wayland_sessions() {
        assert!(session_is_wayland(true, Some("x11")));
        assert!(session_is_wayland(false, Some("WAYLAND")));
        assert!(!session_is_wayland(false, Some("x11")));
        assert!(!session_is_wayland(false, None));
    }

    #[test]
    fn empty_wake_request_is_a_noop() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("current-thread runtime");
        let result = runtime
            .block_on(push_wake_invites(Vec::new()))
            .expect("empty wake request");
        assert!(result.names.is_empty());
        assert_eq!(result.adverts, 0);
    }
}
