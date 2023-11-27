#[derive(Debug, Clone)]
pub struct Config {
    /// Port the webserver should bind to
    pub app_port: u16,
    /// directory that is exposed by talky
    pub base_dir: String,
}

impl Config {
    pub fn init() -> Config {
        let app_port = std::env::var("APP_PORT").unwrap_or("3000".to_owned());
        let base_dir = std::env::var("BASE_DIR").expect("the BASE_DIR is required");

        Config {
            app_port: app_port.parse::<u16>().unwrap_or(3000),
            base_dir,
        }
    }
}
