#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_loadbalancer_config() {
        let config_value: bool = String::from("false").parse().unwrap();
        assert_eq!(config_value, false);
        let config_value: bool = String::from("true").parse().unwrap();
        assert_eq!(config_value, true);

        let config_value: bool = String::from("False").to_lowercase().parse().unwrap();
        assert_eq!(config_value, false);

    }
}
