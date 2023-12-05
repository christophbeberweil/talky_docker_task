use include_dir::{include_dir, Dir};

#[derive(Debug, Clone)]
pub struct Config {
    /// Port the webserver should bind to
    pub app_port: u16,
    /// directory that is exposed by talky
    pub base_dir: String,
    pub default_template: String,
}

impl Config {
    pub fn init() -> Config {
        static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

        let app_port = std::env::var("APP_PORT").unwrap_or("3000".to_owned());
        let base_dir = std::env::var("BASE_DIR").expect("the BASE_DIR is required");

        let default_template = STATIC_DIR
            .get_file("index.html")
            .expect("that default index.html exists")
            .contents_utf8()
            .expect("that the default index.html is readable")
            .to_owned();

        Config {
            app_port: app_port.parse::<u16>().unwrap_or(3000),
            base_dir,
            default_template,
        }
    }
}
