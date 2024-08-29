use std::{fs::{self}, path::PathBuf, str::FromStr};

use getset::{Getters, MutGetters, Setters};

use crate::dossier;

use super::{resource_reference::ResourceReference, source::Source, ResourceError};


/// Image resource to manipulate images
#[derive(Debug, Getters, MutGetters, Setters, Clone)]
pub struct ImageResource {

    #[getset(get = "pub", get_mut = "pub", set = "pub")]
    src: Source,

    #[getset(get = "pub", set = "pub")]
    mime_type: Option<String>,

    #[getset(get = "pub", set = "pub")]
    id: Option<ResourceReference>,

    #[getset(get = "pub", set = "pub")]
    caption: Option<String>,

    #[getset(get = "pub", set = "pub")]
    style: Option<String>,
}

impl ImageResource {

    pub fn new(src: Source, mime_type: Option<String>, id: Option<ResourceReference>, caption: Option<String>, style: Option<String>) -> Self {

        Self {
            src,
            mime_type,
            id,
            caption,
            style
        }
    }

    pub fn inferring_id_if_not_set(mut self, document_name: &impl ToString) -> Result<Self, ResourceError> {

        if self.id.is_none() {
            
            if let Some(ref caption) = self.caption {

                self.id = Some(ResourceReference::of_internal_from_without_sharp(caption, Some(document_name))?)

            } else {

                match &self.src {
                    Source::Remote { url } => self.id = Some(ResourceReference::of_url(url.as_str())?),
                    Source::Local { path } => self.id = Some(ResourceReference::of_asset(path.to_string_lossy().to_string().as_str())?),
                    Source::Base64String { base64: _ } | Source::Bytes { bytes: _ } => todo!(),     // TODO
                }

            }
        }

        Ok(self)
    } 

    /// Infer mime type from image path using `infer` lib.
    /// If `src` is not a path error occurs.
    /// 
    /// `text/xml` is replaced by `image/svg+xml`
    pub fn inferring_mime_type(mut self) -> Result<Self, ResourceError> {

        match &self.src {
            Source::Remote { url } => {
                log::debug!("impossible to infer mime type of url: {}", url);
            },
            Source::Local { path } => {
                let mime_type = infer::get_from_path(path)?;

                if let Some(t) = mime_type {

                    let mut mime_type = t.mime_type().to_string();

                    // work-around svg+xml
                    if mime_type.contains("text/xml") {
                        mime_type = String::from("image/svg+xml");
                    }

                    self.set_mime_type(Some(mime_type));

                    return Ok(self);

                } else {
                    return Err(ResourceError::InvalidResourceVerbose(format!("image {:?} mime type not found", self.src)));
                }
            },
            Source::Base64String { base64: _ } | Source::Bytes { bytes: _ } => todo!(),
        }

        Ok(self)
    }

    /// Call `inferring_mime_type`, but if this returns an error nothing will be done
    pub fn inferring_mime_type_or_nothing(self) -> Self {
        let backup = self.clone();

        match self.inferring_mime_type() {
            Ok(ok) => ok,
            Err(err) => {
                log::warn!("wrong inferring MIME type of image ({:?}): {}", backup.src(), err.to_string());

                backup
            },
        }
    }

    /// Elaborate `src` path if it is relative appending if it doesn't exist dossier `assets` directory
    pub fn elaborating_relative_path_as_dossier_assets(mut self, base_location: &PathBuf) -> Self {

        match &self.src {
            Source::Local { path } => {

                let mut base_location: PathBuf = base_location.clone();
        
                if !base_location.is_dir() {
                    base_location = PathBuf::from(base_location.parent().unwrap());
                }
        
                if path.is_relative() {

                    let mut path = base_location.join(path);
        
                    if !path.exists() {
        
                        let image_file_name = path.file_name().unwrap();
        
                        path = base_location.join(dossier::ASSETS_DIR).join(dossier::IMAGES_DIR).join(image_file_name);
        
                        if !path.exists() {

                            log::warn!("image src path {:?} not found also with images dossier assets path, try canonicalize path", path);

                            if let Ok(src) = fs::canonicalize(path.clone()) {

                                log::info!("canonicalizing ok: {:?} -> {:?}", path, src);

                                path = src;
                            }
                        }
                    }
        
                    self.set_src(Source::Local { path });
                }
            },
            _ => (),
        }

        self
    }

}

impl FromStr for ImageResource {
    type Err = ResourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Source::from_str(s)?, None, None, None, None).inferring_mime_type_or_nothing())
    }
}

#[cfg(test)]
mod test {

}