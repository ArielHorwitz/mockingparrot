#[cfg(test)]
mod config_tests {
    use mockingparrot::config::get_config_from_file;
    use std::path::Path;

    #[test]
    fn template() {
        let template_file = Path::new("config.template.toml");
        get_config_from_file(template_file).expect("parse template config file");
    }
}
