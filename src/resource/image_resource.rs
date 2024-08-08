use std::{fs::{self, File}, io::Read, path::PathBuf, str::FromStr};

use base64::{engine::general_purpose::STANDARD, Engine};
use getset::{Getters, Setters};
use oxipng::Options;

use crate::dossier;

use super::{source::Source, ResourceError};


/// Image resource to manipulate images
#[derive(Debug, Getters, Setters, Clone)]
pub struct ImageResource {

    #[getset(get = "pub", set = "pub")]
    src: Source,

    #[getset(get = "pub", set = "pub")]
    mime_type: Option<String>,

    #[getset(get = "pub", set = "pub")]
    caption: Option<String>,

    #[getset(get = "pub", set = "pub")]
    label: Option<String>,
}

impl ImageResource {

    pub fn new(src: Source, caption: Option<String>, label: Option<String>) -> Self {

        Self {
            src,
            mime_type: None,
            caption,
            label
        }
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
        }

        Ok(self)
    }

    /// Call `inferring_mime_type`, but if this returns an error nothing will be done
    pub fn inferring_mime_type_or_nothing(self) -> Self {
        let backup = self.clone();

        match self.inferring_mime_type() {
            Ok(ok) => ok,
            Err(err) => {
                log::warn!("{}", err.to_string());

                backup
            },
        }
    }

    /// Elaborate `src` path if it is relative appending if it doesn't exist dossier `assets` directory
    pub fn elaborating_relative_path_as_dossier_assets(mut self, base_location: &PathBuf) -> Self {

        match &self.src {
            Source::Remote { url: _ } => (),
            Source::Local { path } => {

                let mut base_location: PathBuf = base_location.clone();
        
                if !base_location.is_dir() {
                    base_location = PathBuf::from(base_location.parent().unwrap());
                }
        
                if path.is_relative() {

                    let mut path = base_location.join(path);
        
                    if !path.exists() {
        
                        log::debug!("{:?} not found, try adding images directory path", path);
        
                        let image_file_name = path.file_name().unwrap();
        
                        path = base_location.join(dossier::ASSETS_DIR).join(dossier::IMAGES_DIR).join(image_file_name);
        
                        if !path.exists() {
                            if let Ok(src) = fs::canonicalize(path.clone()) {
                                path = src;
                            }
                        }
                    }
        
                    self.set_src(Source::Local { path });
                }
            },
        }

        self
    }

    /// Encode image in base64
    pub fn to_base64(&self, compression: bool) -> Result<String, ResourceError> {

        let buffer = self.to_vec_u8()?;

        if compression {

            let original_log_level = log::max_level();
            log::set_max_level(log::LevelFilter::Warn);

            let options = Options::max_compression();

            let optimized_png = oxipng::optimize_from_memory(&buffer, &options);

            log::set_max_level(original_log_level);
    
            match optimized_png {
                Ok(image) => return Ok(STANDARD.encode(image)),
                Err(err) => return Err(ResourceError::ElaborationError(format!("image compression error: {}", err)))
            }

        } else {

            Ok(STANDARD.encode(buffer))
        }
    }

    pub fn to_vec_u8(&self) -> Result<Vec<u8>, ResourceError> {

        match &self.src {
            Source::Remote { url } => return Err(ResourceError::WrongElaboration(format!("url '{}' cannot be parsed into bytes", url))),
            Source::Local { path } => {

                let mut image_file = File::open(path.clone())?;
                let mut raw_image: Vec<u8> = Vec::new();
                image_file.read_to_end(&mut raw_image)?;

                return Ok(raw_image)
            },
        }
    }
}

impl FromStr for ImageResource {
    type Err = ResourceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(Source::from_str(s)?, None, None).inferring_mime_type()?)
    }
}

#[cfg(test)]
mod test {

}