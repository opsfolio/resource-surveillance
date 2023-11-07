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
}
