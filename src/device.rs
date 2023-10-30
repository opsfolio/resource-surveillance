pub struct Device {
    name: String,
}

impl Device {
    pub fn new() -> Device {
        let name = hostname::get()
            .map(|os_str| {
                os_str
                    .into_string()
                    .unwrap_or_else(|_| "unknown".to_owned())
            })
            .unwrap_or_else(|_| "unknown".to_owned());

        Device { name }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
