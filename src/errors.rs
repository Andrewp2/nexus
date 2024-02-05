use core::fmt;
use std::{error::Error, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NexusError {
    PasswordsNotMatching,
    DisplayNameInappropriate,
    CouldNotHashPassword,
    GenericDynamoServiceError,
    BadUsernameEmailCombination,
    GenericSesError,
    CouldNotFindRowWithThatEmail,
    EmailVerificationUuidNotFound,
    EmailNotFoundLogin,
    IncorrectPassword,
    InvalidSession,
    EmailAlreadyInUse,
    AccountNotVerified,
    BadEmailAddress,
    #[serde(other)]
    Unhandled,
}

impl FromStr for NexusError {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::de::from_str(s)
            .map_err(|e| "Could not deserialize string into NexusError".to_string())
    }
}

impl fmt::Display for NexusError {
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

