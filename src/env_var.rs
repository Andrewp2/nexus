use std::env;

pub fn get_table_name() -> &'static str {
    match env::var("STAGE") {
        Ok(stage) => match stage.as_str() {
            "prod" => return "Users",
            "staging" => "Users-staging",
            "dev" => "Users-dev",
            _ => panic!("STAGE environment variable not correct"),
        },
        Err(_) => {
            panic!("Cannot get STAGE");
        }
    }
}

pub fn get_stripe_webhook_signature() -> String {
    match env::var("STRIPE_WEBHOOK_SIGNATURE") {
        Ok(s) => s,
        Err(_) => {
            panic!("Cannot get STRIPE_WEBHOOK_SIGNATURE");
        }
    }
}

