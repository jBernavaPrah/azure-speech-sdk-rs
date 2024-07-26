use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Device {
    pub(crate) os: Os,
    pub(crate) system: System,
}

impl Default for Device {
    fn default() -> Self {
        Device {
            os: Os::default(),
            system: System::unknown(),
        }
    }
}

impl Device {
    /// Overwrite the System information.
    /// This information is taken automatically from the system. But you can overwrite it.
    pub fn set_system(mut self, system: System) -> Self {
        self.system = system;
        self
    }

    /// Overwrite the OS information.
    /// This information is taken automatically from the system. But you can overwrite it.
    pub fn set_os(mut self, os: Os) -> Self {
        self.os = os;
        self
    }
}


impl System {
    #[allow(missing_docs)]
    pub fn unknown() -> Self {
        Self {
            name: "Unknown".to_string(),
            build: "Unknown".to_string(),
            version: "Unknown".to_string(),
            lang: "Unknown".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// OS used to stream audio.
pub struct Os {
    #[allow(missing_docs)]
    pub platform: String,
    #[allow(missing_docs)]
    pub name: String,
    #[allow(missing_docs)]
    pub version: String,
}

impl Default for Os {
    fn default() -> Self {
        let os = os_info::get();
        Os {
            version: os.version().to_string(),
            name: os.os_type().to_string(),
            platform: os.to_string(),
        }
    }
}

impl Os {
    #[allow(missing_docs)]
    pub fn unknown() -> Self {
        Self {
            version: "Unknown".to_string(),
            name: "Unknown".to_string(),
            platform: "Unknown".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// System used to stream audio.
/// This is used by Azure service to provide better results.
pub struct System {
    #[allow(missing_docs)]
    pub name: String,
    #[allow(missing_docs)]
    pub version: String,
    #[allow(missing_docs)]
    pub build: String,
    #[allow(missing_docs)]
    pub lang: String,
}

impl Default for System {
    fn default() -> Self {
        System {
            name: env!("CARGO_PKG_NAME").to_string(),
            build: "rust".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            lang: "rust".to_string(),
        }
    }
}
