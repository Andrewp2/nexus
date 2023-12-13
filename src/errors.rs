use core::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Result;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NexusServerError {
    PasswordsNotMatching,
    DisplayNameInappropriate,
    CouldNotHashPassword,
    GenericDynamoServiceError,
    BadUsernameEmailCombination,
    GenericSesError,
    CouldNotFindRowWithThatEmail,
    EmailVerificationUuidNotFound,
    EmailNotFoundLogin,
    #[serde(other)]
    Unhandled,
}

impl fmt::Display for NexusServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(o) => write!(f, "{}", o),
            Err(err) => {
                log::error!("Unable to generate error {}", err);
                write!(f, "Unable to generate error")
            }
        }
    }
}

