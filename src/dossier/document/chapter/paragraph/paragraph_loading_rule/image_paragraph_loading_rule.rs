use std::str::FromStr;
use once_cell::sync::Lazy;
use regex::Regex;
use super::ParagraphLoadingRule;
use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, dossier::document::chapter::paragraph::{image_paragraph::{ImageParagraph, ImageParagraphContent, MultiImage}, Paragraph}, loader::{loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}, LoadError}, resource::{image_resource::ImageResource, resource_reference::ResourceReference, source::Source, ResourceError}};


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

    fn load_single_image(raw_content: &str, _codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<ImageResource, LoadError> {
        
        let captures = FIND_SINGLE_IMAGE_REGEX.captures(raw_content);

        if let Some(captures) = captures {

            if let Some(caption) = captures.get(1) {

                let caption = caption.as_str().to_string();

                if let Some(src) = captures.get(3) {

                    let src = Source::from_str(src.as_str())?;

                    let id: ResourceReference;

                    if let Some(_id) = captures.get(2) {

                        id = ResourceReference::of_internal_from_without_sharp(_id.as_str(), configuration_overlay.document_name().as_ref())?;

                    } else {

                        id = ResourceReference::of(caption.as_str(), configuration_overlay.document_name().as_ref())?;
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

                    return Ok(image);
                }
            }
        }

        Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
    }

    fn load_abridged_image(raw_content: &str, _codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<ImageResource, LoadError> {

        let captures = FIND_ABRIDGED_IMAGE_REGEX.captures(raw_content);

        if let Some(captures) = captures {

            if let Some(src) = captures.get(1) {

                let src = Source::from_str(src.as_str())?;

                let id: Option<ResourceReference>;

                if let Some(_id) = captures.get(2) {

                    id = Some(ResourceReference::of_internal_from_without_sharp(_id.as_str(), configuration_overlay.document_name().as_ref())?);

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

                return Ok(image);
            }
        }

        Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
    }

    fn load_multi_image(raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<MultiImage, LoadError> {
        let captures = FIND_MULTI_IMAGE_REGEX.captures(raw_content);

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

                let maybe_abridged_image = Self::load_abridged_image(raw_image_line, codex, configuration, configuration_overlay.clone());
                let maybe_single_image = Self::load_single_image(raw_image_line, codex, configuration, configuration_overlay.clone());
                
                // TODO: this is impossible because a multi line image must be written on more than one line
                let maybe_multi_image = Self::load_multi_image(raw_image_line, codex, configuration, configuration_overlay.clone());

                if maybe_single_image.is_err() && maybe_abridged_image.is_err() && maybe_multi_image.is_err() {
                    return Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(format!("{} must be an image", raw_image_line))))
                }

                if let Ok(image) = maybe_abridged_image {

                    images.push((
                        ImageParagraphContent::AbridgedImage(image),
                        align_self.to_string()
                    ));

                    continue;
                }

                if let Ok(image) = maybe_single_image {

                    images.push((
                        ImageParagraphContent::SingleImage(image),
                        align_self.to_string()
                    ));

                    continue;
                }

                if let Ok(multi_image) = maybe_multi_image {

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


            return Ok(multi_image)
        }

        Err(LoadError::ResourceError(ResourceError::InvalidResourceVerbose(String::from("not valid image paragraph provided"))))
    }
}


impl ParagraphLoadingRule for ImageParagraphLoadingRule {
    fn load(&self, raw_content: &str, codex: &Codex, configuration: &LoaderConfiguration, configuration_overlay: LoaderConfigurationOverLay) -> Result<Box<dyn Paragraph>, LoadError> {
        match *self {
            Self::SingleImage => Ok(Box::new(ImageParagraph::new(
                raw_content.to_string(),
                ImageParagraphContent::SingleImage(Self::load_single_image(raw_content, codex, configuration, configuration_overlay.clone())?)
            ))),
            Self::AbridgedImage => Ok(Box::new(ImageParagraph::new(
                raw_content.to_string(),
                ImageParagraphContent::AbridgedImage(Self::load_abridged_image(raw_content, codex, configuration, configuration_overlay.clone())?)
            ))),
            Self::MultiImage => Ok(Box::new(ImageParagraph::new(
                raw_content.to_string(),
                ImageParagraphContent::MultiImage(Self::load_multi_image(raw_content, codex, configuration, configuration_overlay.clone())?)
            ))),
        }
    }
}


#[cfg(test)]
mod test {

    use std::path::PathBuf;

    use crate::{codex::Codex, dossier::document::chapter::paragraph::image_paragraph::ImageParagraphContent, loader::loader_configuration::{LoaderConfiguration, LoaderConfigurationOverLay}};
    use super::ImageParagraphLoadingRule;


    #[test]
    fn load_single_image_from_url() {

        let src = "https://en.wikipedia.org/wiki/Main_Page";
        let caption = "This is a *caption*";

        let nmd_text = format!("![{}]({})", caption, src);

        let codex = Codex::of_html();

        let image = ImageParagraphLoadingRule::load_single_image(&nmd_text, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();
    
        assert_eq!(image.src().to_string(), src);
        assert_eq!(image.caption().as_ref().unwrap(), caption);
    
    }

    #[test]
    fn load_single_image_from_path() {

        let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources").join("wikipedia-logo.png").to_string_lossy().to_string();
        let caption = "This is a *caption*";

        let nmd_text = format!("![{}]({})", caption, src);

        let codex = Codex::of_html();

        let image = ImageParagraphLoadingRule::load_single_image(&nmd_text, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();
    
        assert_eq!(image.src().to_string(), src);
        assert_eq!(image.caption().as_ref().unwrap(), caption);
    
    }

    #[test]
    fn load_abridged_image() {

        let src = "https://en.wikipedia.org/wiki/Main_Page";

        let nmd_text = format!("![({})]", src);

        let codex = Codex::of_html();

        let image = ImageParagraphLoadingRule::load_abridged_image(&nmd_text, &codex, &LoaderConfiguration::default(), LoaderConfigurationOverLay::default()).unwrap();
    
        assert_eq!(image.src().to_string(), src);
    
    }

    #[test]
    fn load_multi_image() {

        let src = "https://en.wikipedia.org/wiki/Main_Page";

        let nmd_text = concat!(
            "!!:space-between:[[\n",
            ":center:![(https://en.wikipedia.org/wiki/Main_Page)]#image-7{width:70%}\n",
            "![Wikipedia](https://en.wikipedia.org/wiki/Main_Page){width:45%;margin:0;}\n",
            "]]"
        );

        let codex = Codex::of_html();

        let mut loader_configuration_overlay = LoaderConfigurationOverLay::default();
        loader_configuration_overlay.set_document_name(Some("test".to_string()));

        let multi_image = ImageParagraphLoadingRule::load_multi_image(&nmd_text, &codex, &LoaderConfiguration::default(), loader_configuration_overlay).unwrap();
    
        assert_eq!(multi_image.alignment, "space-between");

        let (image1, align1) = &multi_image.images[0];
        let (image2, align2) = &multi_image.images[1];

        assert_eq!(align1, "center");
        assert_eq!(align2, "center");

        if let ImageParagraphContent::AbridgedImage(image) = image1 {

            assert_eq!(image.src().to_string(), src);

        } else {
            panic!()
        }

        if let ImageParagraphContent::SingleImage(image) = image2 {

            assert_eq!(image.src().to_string(), src);

        } else {
            panic!()
        }
    
    }
}