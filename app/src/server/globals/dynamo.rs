use super::env_var::get_table_name;
use crate::errors::NexusError;
use aws_sdk_dynamodb::{
    operation::{
        query::builders::QueryFluentBuilder, update_item::builders::UpdateItemFluentBuilder,
    },
    types::AttributeValue,
};
use std::collections::HashMap;

#[cfg(feature = "ssr")]
pub mod constants {
    pub const GAME_NAME_1: &str = "game_1";
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
        pub const EMAIL_VERIFICATION_REQUEST_TIME: &str = "email_verification_request_time";
    }
    pub mod index {
        pub const SESSION_ID_INDEX: &str = "session_id-index";
        pub const EMAIL_VERIFICATION_UUID_INDEX: &str = "email_verification_uuid-index";
    }
}

/// Types of values you can use to query the Users table
pub enum TableKeyType {
    SessionId,
    EmailVerificationUUID,
    Email,
}

pub fn key_type_to_index_string(table_key_type: &TableKeyType) -> String {
    match table_key_type {
        TableKeyType::SessionId => "session_id-index",
        TableKeyType::EmailVerificationUUID => "email_verification_uuid-index",
        TableKeyType::Email => "email",
    }
    .to_string()
}

/// Types of values you can get from querying the Users table
#[derive(Clone, Copy)]
pub enum TableAttributeType {
    DisplayName,
    Email,
    Password,
    GamesBought,
    UserUUID,
    EmailVerified,
    AccountCreationTime,
    SessionId,
    SessionExpiry,
    EmailVerificationUUID,
}

fn attribute_type_to_string_name(o: &TableAttributeType) -> String {
    match o {
        TableAttributeType::DisplayName => "display_name",
        TableAttributeType::Email => "email",
        TableAttributeType::Password => "password",
        TableAttributeType::GamesBought => "games_bought",
        TableAttributeType::UserUUID => "user_uuid",
        TableAttributeType::EmailVerified => "email_verified",
        TableAttributeType::AccountCreationTime => "account_creation_time",
        TableAttributeType::SessionId => "session_id",
        TableAttributeType::SessionExpiry => "session_expiry",
        TableAttributeType::EmailVerificationUUID => "email_verification_uuid",
    }
    .to_string()
}

pub fn parse_string_attribute(
    item: &HashMap<String, AttributeValue>,
    key: &TableAttributeType,
) -> Result<Option<String>, NexusError> {
    item.get(&attribute_type_to_string_name(key))
        .map(|attr| attr.as_s().map(ToString::to_string))
        .transpose()
        .map_err(|e| {
            log::error!("Couldn't get attribute value as string {:?}", e);
            NexusError::Unhandled
        })
}

pub fn parse_list_of_strings_attribute(
    item: &HashMap<String, AttributeValue>,
    key: &TableAttributeType,
) -> Result<Option<Vec<String>>, NexusError> {
    item.get(&attribute_type_to_string_name(key))
        .map(|attr| {
            attr.as_l().map(|l| {
                l.iter()
                    .map(|game| game.as_s().unwrap().clone())
                    .collect::<Vec<String>>()
            })
        })
        .transpose()
        .map_err(|e| {
            log::error!("Couldn't get attribute value as list {:?}", e);
            NexusError::Unhandled
        })
}

pub fn parse_bool_attribute(
    item: &HashMap<String, AttributeValue>,
    key: &TableAttributeType,
) -> Result<Option<bool>, NexusError> {
    item.get(&attribute_type_to_string_name(key))
        .map(|attr| attr.as_bool().copied())
        .transpose()
        .map_err(|e| {
            log::error!("Couldn't get attribute value as boolean {:?}", e);
            NexusError::Unhandled
        })
}

pub fn parse_number_attribute(
    item: &HashMap<String, AttributeValue>,
    key: &TableAttributeType,
) -> Result<Option<i64>, NexusError> {
    item.get(&attribute_type_to_string_name(key))
        .map(|attr| attr.as_n().map(|s| s.parse::<i64>()))
        .transpose()
        .map_err(|e| {
            log::error!("Couldn't get attribute value as number {:?}", e);
            NexusError::Unhandled
        })?
        .transpose()
        .map_err(|e| {
            log::error!("Couldn't parse attribute value string as number {:?}", e);
            NexusError::Unhandled
        })
}

pub fn query_builder(client: &aws_sdk_dynamodb::Client) -> QueryFluentBuilder {
    client.query().limit(1).table_name(get_table_name())
}

pub fn query_key(
    builder: QueryFluentBuilder,
    table_key_value: String,
    table_key_type: TableKeyType,
) -> QueryFluentBuilder {
    let mut b = builder.key_condition_expression("#key = :k");
    let mut attribute_expression_names = HashMap::with_capacity(1);
    attribute_expression_names.insert(
        "#key".to_string(),
        key_type_to_index_string(&table_key_type),
    );
    let mut attribute_expression_values = HashMap::with_capacity(1);
    attribute_expression_values.insert(":k".to_string(), AttributeValue::S(table_key_value));
    b = b.set_expression_attribute_values(Some(attribute_expression_values));
    b = b.set_index_name(match table_key_type {
        TableKeyType::SessionId | TableKeyType::EmailVerificationUUID => {
            Some(key_type_to_index_string(&TableKeyType::SessionId))
        }
        TableKeyType::Email => None,
    });
    b
}

pub fn query_setup(
    client: &aws_sdk_dynamodb::Client,
    table_key_value: String,
    table_key_type: TableKeyType,
) -> QueryFluentBuilder {
    query_key(query_builder(client), table_key_value, table_key_type)
}

pub fn update_setup(client: &aws_sdk_dynamodb::Client, email: String) -> UpdateItemFluentBuilder {
    client
        .update_item()
        .table_name(get_table_name())
        .key(constants::table_attributes::EMAIL, AttributeValue::S(email))
}
