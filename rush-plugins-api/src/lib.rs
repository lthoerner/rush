use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InitHookParams<'a> {
    pub rush_version: &'a str,
}
