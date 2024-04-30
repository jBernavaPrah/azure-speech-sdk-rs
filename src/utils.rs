pub(crate) fn get_azure_hostname_from_region(region: &String) -> String {
    if region.contains("china") { return String::from(".azure.cn"); }
    if region.to_lowercase().starts_with("usgov") { return String::from(".azure.us"); }

    return String::from(".microsoft.com");
}

#[cfg(test)]
mod tests {
    use crate::utils::get_azure_hostname_from_region;

    #[test]
    fn test_get_azure_hostname_from_region() {
        assert_eq!(get_azure_hostname_from_region(&String::from("fallback")), String::from(".microsoft.com"));
        assert_eq!(get_azure_hostname_from_region(&String::from("chinaeast")), String::from(".azure.cn"));
        assert_eq!(get_azure_hostname_from_region(&String::from("usgovwest")), String::from(".azure.us"));
    }
}