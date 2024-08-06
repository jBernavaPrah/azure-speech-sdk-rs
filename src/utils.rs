pub(crate) fn get_azure_hostname_from_region(region: &str) -> &str {
    if region.contains("china") { return ".azure.cn"; }
    if region.to_lowercase().starts_with("usgov") { return ".azure.us"; }

    ".microsoft.com"
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_azure_hostname_from_region() {
        assert_eq!(get_azure_hostname_from_region("fallback"), ".microsoft.com");
        assert_eq!(get_azure_hostname_from_region("chinaeast"), ".azure.cn");
        assert_eq!(get_azure_hostname_from_region("usgovwest"), ".azure.us");
    }
}