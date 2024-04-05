use std::fmt::Display;
use std::str::FromStr;
use crate::msnp::error::CommandError;
use crate::msnp::notification::models::endpoint_guid::EndpointGuid;
use crate::shared::traits::ParseStr;

pub struct EndpointId {
    email_addr: String,
    endpoint_guid: Option<EndpointGuid>
}

impl Display for EndpointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.email_addr)?;

        if let Some(endpoint_guid) = self.endpoint_guid.as_ref() {
            write!(f, ";{}", endpoint_guid)?;
        }

        Ok(())

    }
}

impl FromStr for EndpointId {
    type Err = CommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.split(';').collect();
        let email_addr = split[0].to_string();

        let endpoint_guid = if split.len() >= 2 {
            Some(EndpointGuid::try_parse_str(split[1])?)
        } else {
            None
        };

        Ok(EndpointId {email_addr, endpoint_guid})
    }
}
