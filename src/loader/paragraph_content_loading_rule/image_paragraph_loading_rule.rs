use std::{str::FromStr, sync::{Arc, RwLock}};

use once_cell::sync::Lazy;
use regex::Regex;

use super::ParagraphContentLoadingRule;
use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, dossier::document::chapter::paragraph::{image_paragraph::{ImageParagraph, ImageParagraphContent, MultiImage}, list_paragraph::ListParagraph, ParagraphTrait}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError}, resource::{image_resource::ImageResource, resource_reference::ResourceReference, source::Source, ResourceError}};


static FIND_SINGLE_IMAGE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&StandardParagraphModifier::Image.modifier_pattern()).unwrap());
static FIND_ABRIDGED_IMAGE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&StandardParagraphModifier::AbridgedImage.modifier_pattern()).unwrap());
static FIND_MULTI_IMAGE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&StandardParagraphModifier::MultiImage.modifier_pattern()).unwrap());
static ALIGN_ITEM_PATTERN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(ALIGN_ITEM_PATTERN).unwrap());

const DEFAULT_MULTI_IMAGE_ALIGNMENT: &str = "normal";
const DEFAULT_ALIGN_SELF: &str = "center";
const ALIGN_ITEM_PATTERN: &str = r":([\w-]*):";


#[derive(Debug)]
pub enum ImageParagraphLoadingRule {
    SingleImage,
    AbridgedImage,
    MultiImage,
}


impl ImageParagraphLoadingRule {

    fn load_single_image(raw_content: &str, _codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: Arc<RwLock<LoaderConfigurationOverLay>>) -> Result<Option<ImageResource>, LoadError> {
        
        let captures = FIND_SINGLE_IMAGE_REGEX.captures(raw_content);

        if let Some(captures) = captures {

            if let Some(caption) = captures.get(1) {

                let caption = caption.as_str().to_string();

                if let Some(src) = captures.get(3) {

                    let src = Source::from_str(src.as_str())?;

                    let id: ResourceReference;

                    if let Some(_id) = captures.get(2) {

                        id = ResourceReference::of_internal_from_without_sharp(_id.as_str(), configuration_overlay.read().unwrap().document_name().as_ref()).unwrap();

                    } else {

                        id = ResourceReference::of(caption.as_str(), configuration_overlay.read().unwrap().document_name().as_ref()).unwrap();
                    }

                    let style: Option<String>;

                    if let Some(_style) = captures.get(4) {
                        style = Some(String::from(_style.as_str()));
                    } else {
                        style = None;
                    }

                    let image = ImageResource::new(src, None, Some(id), Some(caption), style)
                                                                .elaborating_relative_path_as_dossier_assets(configuration.input_location())
                                                                .inferring_mime_type_or_nothing();

                    return Ok(Some(image));
                }
            }

            return Ok(None);
        }

        Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
    }

    fn load_abridged_image(raw_content: &str, _codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: Arc<RwLock<LoaderConfigurationOverLay>>) -> Result<Option<ImageResource>, LoadError> {

        let captures = FIND_ABRIDGED_IMAGE_REGEX.captures(raw_content);

        if let Some(captures) = captures {

            if let Some(src) = captures.get(3) {

                let src = Source::from_str(src.as_str())?;

                let id: Option<ResourceReference>;

                if let Some(_id) = captures.get(2) {

                    id = Some(ResourceReference::of_internal_from_without_sharp(_id.as_str(), configuration_overlay.read().unwrap().document_name().as_ref()).unwrap());

                } else {

                    id = None;
                }

                let style: Option<String>;

                if let Some(_style) = captures.get(3) {
                    style = Some(String::from(_style.as_str()));
                } else {
                    style = None;
                }

                let image = ImageResource::new(src, None, id, None, style)
                                                            .elaborating_relative_path_as_dossier_assets(configuration.input_location())
                                                            .inferring_mime_type_or_nothing();

                return Ok(Some(image));
            }

            return Ok(None);
        }

        Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
    }

    fn load_multi_image(raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: Arc<RwLock<LoaderConfigurationOverLay>>) -> Result<Option<MultiImage>, LoadError> {
        let captures = FIND_SINGLE_IMAGE_REGEX.captures(raw_content);

        if let Some(captures) = captures {

            let multi_image_alignment: String;

            if let Some(align) = captures.get(1) {
                multi_image_alignment = String::from(align.as_str());
            } else {
                multi_image_alignment = DEFAULT_MULTI_IMAGE_ALIGNMENT.to_string();
            }

            let raw_images = String::from(captures.get(2).unwrap().as_str());

            let mut images: Vec<(ImageParagraphContent, String)> = Vec::new();

            for mut raw_image_line in raw_images.lines() {

                if raw_image_line.trim().is_empty() {
                    continue;
                }

                let align_self_captures = ALIGN_ITEM_PATTERN_REGEX.captures(raw_image_line);

                let align_self = match align_self_captures {
                    Some(ai) => {
                        raw_image_line = raw_image_line.strip_prefix(ai.get(0).unwrap().as_str()).unwrap();

                        ai.get(1).unwrap().as_str()
                    },
                    None => DEFAULT_ALIGN_SELF
                };

                if let Some(image) = Self::load_single_image(raw_image_line, codex, configuration, configuration_overlay.clone())? {

                    images.push((
                        ImageParagraphContent::SingleImage(image),
                        align_self.to_string()
                    ));

                    continue;
                }

                if let Some(image) = Self::load_abridged_image(raw_image_line, codex, configuration, configuration_overlay.clone())? {

                    images.push((
                        ImageParagraphContent::AbridgedImage(image),
                        align_self.to_string()
                    ));

                    continue;
                }

                if let Some(multi_image) = Self::load_multi_image(raw_image_line, codex, configuration, configuration_overlay.clone())? {

                    images.push((
                        ImageParagraphContent::MultiImage(multi_image),
                        align_self.to_string()
                    ));

                    continue;
                }
            }

            let multi_image = MultiImage {
                alignment: multi_image_alignment,
                images,
            };


            return Ok(Some(multi_image))
        }

        Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
    }
}


impl ParagraphContentLoadingRule for ImageParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: Arc<RwLock<LoaderConfigurationOverLay>>) -> Result<Box<dyn ParagraphTrait>, LoadError> {
        match *self {
            Self::SingleImage => {

                if let Some(image) = Self::load_single_image(raw_content, codex, configuration, configuration_overlay.clone())? {

                    return Ok(Box::new(ImageParagraph::new(raw_content.to_string(), ImageParagraphContent::SingleImage(image))))
                }

                Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
            },
            Self::AbridgedImage => {

                if let Some(image) = Self::load_abridged_image(raw_content, codex, configuration, configuration_overlay.clone())? {

                    return Ok(Box::new(ImageParagraph::new(raw_content.to_string(), ImageParagraphContent::AbridgedImage(image))))
                }

                Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
            },
            Self::MultiImage => {

                if let Some(multi_image) = Self::load_multi_image(raw_content, codex, configuration, configuration_overlay.clone())? {

                    return Ok(Box::new(ImageParagraph::new(raw_content.to_string(), ImageParagraphContent::MultiImage(multi_image))))
                }

                Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
            },
        }
    }
}