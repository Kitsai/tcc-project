use serde::{Deserialize, Serialize};

use crate::polygon::error::PolygonCredentialError;

#[derive(Serialize, Deserialize, Debug)]
pub struct PolygonCredentials {
    pub user_name: String,
    pub api_key: String,
    pub secret: String,
}

impl PolygonCredentials {
    const SERVICE_NAME: &'static str = "polygon.codeforces";

    pub fn new(user_name: &str, api_key: &str, secret: &str) -> Self {
        PolygonCredentials {
            user_name: user_name.to_owned(),
            api_key: api_key.to_owned(),
            secret: secret.to_owned(),
        }
    }

    pub fn get(user_name: &str) -> Result<Self, PolygonCredentialError> {
        let entry = keyring::Entry::new(Self::SERVICE_NAME, user_name)?;
        let password = entry.get_password()?;
        if let Some((api_key, secret)) = password.split_once(':') {
            return Ok(PolygonCredentials::new(user_name, api_key, secret));
        }
        Err(PolygonCredentialError::InvalidFormat)
    }
    pub fn save(&self) -> Result<(), PolygonCredentialError> {
        let joined = format!("{}:{}", self.api_key, self.secret);
        let entry = keyring::Entry::new(Self::SERVICE_NAME, &self.user_name)?;
        entry.set_password(&joined)?;

        Ok(())
    }

    pub fn delete(&self) -> Result<(), PolygonCredentialError> {
        let entry = keyring::Entry::new(Self::SERVICE_NAME, &self.user_name)?;
        entry.delete_credential()?;
        Ok(())
    }

    pub fn delete_by_user(user_name: &str) -> Result<(), PolygonCredentialError> {
        let entry = keyring::Entry::new(Self::SERVICE_NAME, user_name)?;
        entry.delete_credential()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use keyring::Entry;

    use crate::polygon::{error::PolygonCredentialError, polygon_credentials::PolygonCredentials};

    #[test]
    fn get_should_suceed_with_valid_credentials() {
        let creds = PolygonCredentials {
            user_name: "test_user_get_valid".to_owned(),
            api_key: "Strong".to_owned(),
            secret: "password".to_owned(),
        };
        creds.save().unwrap();

        let res = PolygonCredentials::get(&creds.user_name);

        let value = res.unwrap();

        assert_eq!("Strong", value.api_key);
        assert_eq!("password", value.secret);

        creds.delete().unwrap();
    }

    #[test]
    fn get_should_fail_with_invalid_credential() {
        let user_name = "test_user_get_invalid";
        let entry = Entry::new(PolygonCredentials::SERVICE_NAME, user_name).unwrap();
        entry.set_password("Password").unwrap();

        let res = PolygonCredentials::get(user_name);

        assert!(matches!(res, Err(PolygonCredentialError::InvalidFormat)));
        PolygonCredentials::delete_by_user(user_name).unwrap();
    }
}
