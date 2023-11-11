use serde_json::json;
use sysinfo::{System, SystemExt};

pub struct Device {
    pub name: String,
    pub boundary: Option<String>,
}

impl Device {
    pub fn new(boundary: Option<String>) -> Device {
        let name = hostname::get()
            .map(|os_str| {
                os_str
                    .into_string()
                    .unwrap_or_else(|_| "unknown".to_owned())
            })
            .unwrap_or_else(|_| "unknown".to_owned());

        Device { name, boundary }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn state_json(&self) -> String {
        // TODO: support other states (meaning devices with multiple "versions")
        serde_json::to_string_pretty(&json!("SINGLETON")).unwrap()
    }

    pub fn state_sysinfo_json(&self) -> String {
        let mut sys = System::new_all();
        sys.refresh_all();

        serde_json::to_string_pretty(&sys).unwrap()
    }
}
