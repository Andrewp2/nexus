use leptos::{server, ServerFnError};

use crate::errors::NexusError;

/// Contains all the public-facing API calls.
// TODO: Implement rate limiting? Handle on API Gateway instead of here maybe?

/// Logs the given user in
#[server(Login, "/api", "Url", "login")]
pub async fn login(
    email: String,
    password: String,
    #[server(default)] remember: String,
) -> Result<String, ServerFnError<NexusError>> {
    let remember = match remember.as_str() {
        "true" => true,
        "on" => true,
        "false" => false,
        "off" => false,
        "" => false,
        _ => false,
    };
    crate::server::login::login(email, password, remember).await
}

/// Logs the user out
#[server(Logout, "/api", "Url", "logout")]
pub async fn logout() -> Result<(), ServerFnError<NexusError>> {
    //crate::server::logout::logout().await
    return Ok(());
}

/// Server function that signs the user up.
/// Sends an email to the given email address.
#[server(Signup, "/api", "Url", "signup")]
pub async fn signup(
    display_name: String,
    email: String,
    password: String,
    password_confirmation: String,
) -> Result<(), ServerFnError<NexusError>> {
    crate::server::signup::signup(display_name, email, password, password_confirmation).await
}

#[server(VerifyEmail, "/api", "Url", "verify_email")]
pub async fn verify_email(email_uuid: String) -> Result<(), ServerFnError<NexusError>> {
    crate::server::verify_email::verify_email(email_uuid).await
}

#[server(ChangeEmailRequest, "/api", "Url", "change_email_request")]
pub async fn change_email_request(new_email: String) -> Result<(), ServerFnError<NexusError>> {
    crate::server::change_profile::change_email_request(new_email).await
}

#[server(ChangeDisplayName, "/api", "Url", "change_display_name")]
pub async fn change_display_name(
    new_display_name: String,
) -> Result<(), ServerFnError<NexusError>> {
    crate::server::change_profile::change_display_name(new_display_name).await
}

#[server(ChangePassword, "/api", "Url", "change_password")]
pub async fn change_password(new_password: String) -> Result<(), ServerFnError<NexusError>> {
    crate::server::change_profile::change_password(new_password).await
}

#[server(CreateCheckout, "/api", "Url", "create_checkout")]
pub async fn create_checkout() -> Result<String, ServerFnError<NexusError>> {
    crate::server::create_checkout::create_checkout().await
}
