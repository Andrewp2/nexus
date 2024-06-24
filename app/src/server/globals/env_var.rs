pub fn get_table_name() -> &'static str {
    match std::env!("STAGE") {
        "prod" => "Users",
        "staging" => "Users-staging",
        "dev" => "Users-dev",
        _ => panic!("STAGE environment variable was not set to 'prod', 'staging', or 'dev' at compile-time.")
    }
}

pub fn get_host_prefix() -> &'static str {
    if cfg!(debug_assertions) {
        ""
    } else {
        "__Host-"
    }
}
