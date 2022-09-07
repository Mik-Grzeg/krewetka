use super::errors::ImporterError;
pub trait Import {
    fn import(&self) -> Result<Vec<u8>, ImporterError>;
}