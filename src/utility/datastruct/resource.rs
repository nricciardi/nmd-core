pub mod disk_resource;
pub mod cached_disk_resource;


use std::str::FromStr;
use crate::mmo::MultiMediaObjectError;


/// General physical or virtual resource
pub trait Resource: FromStr {

    type LocationType;

    /// write resource content
    fn write(&mut self, content: &str) -> Result<(), MultiMediaObjectError>;

    /// erase content resource
    fn erase(&mut self) -> Result<(), MultiMediaObjectError>;

    /// append resource content
    fn append(&mut self, content: &str) -> Result<(), MultiMediaObjectError>;

    /// read resource content
    fn read(&self) -> Result<String, MultiMediaObjectError>;

    /// return resource content
    fn content(&self) -> Result<String, MultiMediaObjectError> {
        self.read()        
    }

    /// return resource name
    fn name(&self) -> &String;

    /// return embedded location type (e.g. PathBuf for files)
    fn location(&self) -> &Self::LocationType;
}