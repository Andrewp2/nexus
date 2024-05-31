use std::env;

pub fn get_table_name() -> &'static str {
    match env::var("STAGE") {
        Ok(stage) => match stage.as_str() {
            "prod" => "Users",
            "staging" => "Users-staging",
            "dev" => "Users-dev",
            _ => panic!("STAGE environment variable not set to prod, staging, or dev"),
        },
        Err(_) => {
            panic!("Cannot get STAGE environment variable to determine table name");
        }
    }
}

pub fn get_host_prefix() -> &'static str {
    if cfg!(debug_assertions) {
        ""
    } else {
        "__Host-"
    }
}
