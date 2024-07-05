#[cfg(test)]
mod config_tests {
    use mockingparrot::config::Config;
    use std::path::Path;

    #[test]
    fn template() {
        let config_toml = std::fs::read_to_string(Path::new("config.template.toml"))
            .expect("read config template file");
        toml::from_str::<Config>(&config_toml).expect("parse config template file toml");
    }
}
