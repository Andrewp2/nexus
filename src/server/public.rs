use leptos::{server, ServerFnError};

/// Contains all the public-facing API calls.
// TODO: Implement rate limiting? Handle on API Gateway instead of here maybe?

/// Logs the given user in
#[server(Login, "/api")]
pub async fn login(email: String, password: String, remember: bool) -> Result<(), ServerFnError> {
    use super::login::login;
    login(email, password, remember).await
}

/// Logs the user out
#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    use super::logout::logout;
    logout().await
}

/// Server function that signs the user up.
/// Sends an email to the given email address.
#[server(Signup, "/api")]
pub async fn signup(
    display_name: String,
    email: String,
    password: String,
    password_confirmation: String,
) -> Result<(), ServerFnError> {
    use super::signup::signup;
    return signup(display_name, email, password, password_confirmation).await;
}

#[server(VerifyEmail, "/api")]
pub async fn verify_email_on_signup(email_uuid: String) -> Result<(), ServerFnError> {
    use super::verify_email::verify_email_for_signup;
    verify_email_for_signup(email_uuid).await
}

#[server(ChangeEmailRequest, "/api")]
pub async fn change_email_request(new_email: String) -> Result<(), ServerFnError> {
    use super::change_profile::change_email_request;
    change_email_request(new_email).await
}

#[server(ChangeEmailValidation, "/api")]
pub async fn change_email_validation(email_uuid: String) -> Result<(), ServerFnError> {
    use super::change_profile::change_email_validation;
    change_email_validation(email_uuid).await
}

#[server(ChangeDisplayName, "/api")]
pub async fn change_display_name(new_display_name: String) -> Result<(), ServerFnError> {
    use super::change_profile::change_display_name;
    change_display_name(new_display_name).await
}

#[server(ChangePassword, "/api")]
pub async fn change_password(new_password: String) -> Result<(), ServerFnError> {
    use super::change_profile::change_password;
    change_password(new_password).await
}

