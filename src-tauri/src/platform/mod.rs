use std::path::Path;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HostPower {
    pub battery: i64,
    pub charging: bool,
    pub plugged: bool,
    pub battery_available: bool,
}

impl HostPower {
    pub(crate) fn on_power() -> Self {
        Self {
            battery: 100,
            charging: false,
            plugged: true,
            battery_available: false,
        }
    }

    pub(crate) fn line(&self) -> String {
        let how = if self.charging {
            "charging"
        } else if self.plugged {
            "plugged in"
        } else {
            "on battery"
        };
        let host = if cfg!(target_os = "macos") {
            "This Mac"
        } else {
            "This computer"
        };
        format!("{host} is at {}% and {how}.", self.battery)
    }
}

#[cfg_attr(target_os = "linux", allow(dead_code))]
#[derive(Clone, Debug)]
pub(crate) struct WakeTarget {
    pub name: String,
    pub real: Vec<u8>,
    pub frame: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct WakeOutcome {
    pub names: Vec<String>,
    pub adverts: u32,
    pub payloads: u32,
    pub matched: u32,
}

#[derive(Debug)]
pub(crate) enum PlatformError {
    Unsupported(String),
    Unavailable(String),
    #[cfg(target_os = "macos")]
    PermissionDenied(String),
    Failed(String),
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(message) | Self::Unavailable(message) | Self::Failed(message) => {
                f.write_str(message)
            }
            #[cfg(target_os = "macos")]
            Self::PermissionDenied(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for PlatformError {}

#[derive(Clone)]
pub(crate) struct PlatformServices {
    #[cfg(target_os = "macos")]
    app: tauri::AppHandle,
}

impl PlatformServices {
    pub(crate) fn new(app: tauri::AppHandle) -> Self {
        #[cfg(target_os = "macos")]
        {
            Self { app }
        }
        #[cfg(target_os = "linux")]
        {
            let _ = app;
            Self {}
        }
    }

    pub(crate) fn initialize(&self) {
        #[cfg(target_os = "macos")]
        macos::initialize(&self.app);
        #[cfg(target_os = "linux")]
        linux::initialize();
    }

    pub(crate) fn host_power(&self) -> HostPower {
        #[cfg(target_os = "macos")]
        {
            macos::host_power()
        }
        #[cfg(target_os = "linux")]
        {
            linux::host_power()
        }
    }

    pub(crate) fn lock_session(&self) -> Result<(), PlatformError> {
        #[cfg(target_os = "macos")]
        {
            macos::lock_session()
        }
        #[cfg(target_os = "linux")]
        {
            linux::lock_session()
        }
    }

    pub(crate) fn open_path(&self, path: &Path) -> Result<(), PlatformError> {
        #[cfg(target_os = "macos")]
        {
            macos::open_path(path)
        }
        #[cfg(target_os = "linux")]
        {
            linux::open_path(path)
        }
    }

    pub(crate) fn read_clipboard_text(&self) -> Result<String, PlatformError> {
        #[cfg(target_os = "macos")]
        {
            macos::read_clipboard_text()
        }
        #[cfg(target_os = "linux")]
        {
            linux::read_clipboard_text()
        }
    }

    pub(crate) fn write_clipboard_text(&self, text: &str) -> Result<(), PlatformError> {
        #[cfg(target_os = "macos")]
        {
            macos::write_clipboard_text(text)
        }
        #[cfg(target_os = "linux")]
        {
            linux::write_clipboard_text(text)
        }
    }

    pub(crate) fn show_banner(
        &self,
        app: String,
        title: String,
        body: String,
        key: String,
        pkg: String,
        icon: Option<Vec<u8>>,
        on_unavailable: impl FnOnce() + Send + 'static,
    ) {
        #[cfg(target_os = "macos")]
        macos::show_banner(&self.app, app, title, body, key, pkg, icon, on_unavailable);
        #[cfg(target_os = "linux")]
        linux::show_banner(app, title, body, key, pkg, icon, on_unavailable);
    }

    pub(crate) async fn push_wake_invites(
        &self,
        targets: Vec<WakeTarget>,
    ) -> Result<WakeOutcome, PlatformError> {
        #[cfg(target_os = "macos")]
        {
            macos::push_wake_invites(targets).await
        }
        #[cfg(target_os = "linux")]
        {
            linux::push_wake_invites(targets).await
        }
    }

    pub(crate) fn ensure_camera_publisher(&self) {
        #[cfg(target_os = "macos")]
        macos::ensure_camera_publisher();
        #[cfg(target_os = "linux")]
        linux::ensure_camera_publisher();
    }

    pub(crate) fn set_camera_published(&self, on: bool) -> Result<(), PlatformError> {
        #[cfg(target_os = "macos")]
        {
            macos::set_camera_published(on)
        }
        #[cfg(target_os = "linux")]
        {
            linux::set_camera_published(on)
        }
    }
}

pub(crate) fn prepare_launch() {
    #[cfg(target_os = "macos")]
    macos::prepare_launch();
}

pub(crate) fn default_computer_name() -> String {
    #[cfg(target_os = "macos")]
    {
        macos::computer_name()
    }
    #[cfg(target_os = "linux")]
    {
        linux::computer_name()
    }
}
