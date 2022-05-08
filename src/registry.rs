use crate::errors::TowError;

// TODO this is just a template for now
// will be implemented via Github and Gitlab integrations
pub trait Registry {
    fn check_for_update(&self, name: String) -> Result<String, TowError>;
    fn get_releases(&self, name: String) -> Result<Vec<String>, TowError>;
    fn get_url(&self, name: String, version: String) -> Result<String, TowError>;
}
