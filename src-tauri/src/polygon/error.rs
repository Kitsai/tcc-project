#[derive(Debug)]
pub enum PolygonCredentialError {
    Keyring(keyring::Error),
    InvalidFormat,
}

impl From<keyring::Error> for PolygonCredentialError {
    fn from(value: keyring::Error) -> Self {
        PolygonCredentialError::Keyring(value)
    }
}
