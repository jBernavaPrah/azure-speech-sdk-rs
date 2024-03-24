pub(crate) trait ToQuery {
    fn to_query(&self) -> String;
}