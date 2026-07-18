pub const fn client_name() -> &'static str {
    "bridgehub"
}

#[cfg(test)]
mod tests {
    #[test]
    fn client_identity_is_stable() {
        assert_eq!(super::client_name(), "bridgehub");
    }
}
