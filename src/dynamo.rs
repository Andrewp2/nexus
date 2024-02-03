#[cfg(feature = "ssr")]
pub mod constants {
    pub mod table_attributes {
        pub const DISPLAY_NAME: &str = "display_name";
        pub const EMAIL: &str = "email";
        pub const PASSWORD: &str = "hashed_password";
        pub const GAMES_BOUGHT: &str = "games_bought";
        pub const USER_UUID: &str = "user_uuid";
        pub const EMAIL_VERIFIED: &str = "email_verified";
        pub const ACCOUNT_CREATION_TIME: &str = "account_creation_time";
        pub const SESSION_ID: &str = "session_id";
        pub const SESSION_EXPIRY: &str = "session_expiry";
        pub const EMAIL_VERIFICATION_UUID: &str = "email_verification_uuid";
    }

    pub mod index {
        pub const GAMES_BOUGHT: &str = "games_bought-index";
        pub const USER_UUID: &str = "user_uuid-index";
        pub const SESSION_ID: &str = "session_id-index";
        pub const EMAIL_VERIFICATION_UUID: &str = "email_verification_uuid-index";
    }
}

