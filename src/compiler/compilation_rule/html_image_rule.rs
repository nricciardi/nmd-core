use std::fs;
use std::sync::RwLock;
use std::{path::PathBuf, sync::Arc};

use build_html::{Container, Html, HtmlContainer};
use log;
use once_cell::sync::Lazy;
use regex::{Regex, Captures};
use url::Url;
use crate::codex::modifier::ModifierIdentifier;
use crate::codex::Codex;
use crate::codex::modifier::standard_paragraph_modifier::StandardParagraphModifier;
use crate::compiler::compilation_configuration::compilation_configuration_overlay::CompilationConfigurationOverLay;
use crate::compiler::compilation_configuration::CompilationConfiguration;
use crate::compiler::compilation_error::CompilationError;
use crate::compiler::compilation_result::CompilationResult;
use crate::compiler::compilation_rule::constants::ESCAPE_HTML;
use crate::compiler::Compiler;
use crate::output_format::OutputFormat;
use crate::resource::resource_reference::ResourceReference;
use crate::resource::{image_resource::ImageResource, remote_resource::RemoteResource};
use crate::utility::text_utility;

use super::CompilationRule;

static ALIGN_ITEM_PATTERN_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(ALIGN_ITEM_PATTERN).unwrap());

const MULTI_IMAGE_PERMITTED_MODIFIER: &'static [StandardParagraphModifier] = &[StandardParagraphModifier::Image, StandardParagraphModifier::AbridgedImage];
const DEFAULT_JUSTIFY_CONTENT: &str = "normal";
const DEFAULT_ALIGN_SELF: &str = "center";
const ALIGN_ITEM_PATTERN: &str = r":([\w-]*):";


#[derive(Debug)]
/// Rule to replace a NMD text based on a specific pattern matching rule
pub struct HtmlImageRule {
    image_modifier_identifier: ModifierIdentifier,
    search_pattern: String,
    search_pattern_regex: Regex,
}

impl HtmlImageRule {
    
    pub fn new(image_modifier_identifier: ModifierIdentifier) -> Self {

        let searching_pattern = Self::get_searching_pattern(&image_modifier_identifier);

        Self {
            image_modifier_identifier,
            search_pattern_regex: Regex::new(&searching_pattern).unwrap(),
            search_pattern: searching_pattern,
        }
    }

    fn get_searching_pattern(image_modifier_identifier: &ModifierIdentifier) -> String {

        if image_modifier_identifier.eq(&StandardParagraphModifier::Image.identifier()) {
            return StandardParagraphModifier::Image.modifier_pattern_with_paragraph_separator()
        }

        if image_modifier_identifier.eq(&StandardParagraphModifier::AbridgedImage.identifier()) {
            return StandardParagraphModifier::AbridgedImage.modifier_pattern_with_paragraph_separator()
        }

        if image_modifier_identifier.eq(&StandardParagraphModifier::MultiImage.identifier()) {
            return StandardParagraphModifier::MultiImage.modifier_pattern_with_paragraph_separator()
        }

        log::error!("'{}' is unsupported image modifier identifier", image_modifier_identifier);

        panic!("unsupported image modifier identifier");
    }

    /// Build html image tag with `<figcaption>` and style
    fn build_html_img(src: &str, alt: Option<&String>, caption: Option<&String>, id: Option<ResourceReference>, img_classes: Vec<&str>, style: Option<String>) -> String {

        let id_attr: String;

        if let Some(id) = id {
            id_attr = format!(r#"id="{}""#, id.build());
        } else {
            id_attr = String::new();
        }

        let html_alt: String;
        let html_caption: String;

        if let Some(a) = alt {
            html_alt = format!(r#"alt="{}""#, a);
        } else {
            html_alt = String::new();
        }

        if let Some(c) = caption {
            html_caption = format!(r#"<figcaption class="image-caption">{}</figcaption>"#, text_utility::replace(c, &ESCAPE_HTML));
        } else {
            html_caption = String::new();
        }

        let style_attr: String;

        if let Some(style) = style {
            style_attr = format!(r#"style="{}""#, style);
        } else {
            style_attr = String::new();
        }

        format!(r#"<figure class="figure" {}>
                    <img src="{}" {} class="{}" {} />
                    {}
                </figure>"#, id_attr, src, html_alt, img_classes.join(" "), style_attr, html_caption)
    }

    fn build_not_embed_remote_img(image: &mut ImageResource, id: Option<ResourceReference>, img_classes: Vec<&str>, figure_style: Option<String>, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<String, CompilationError> {
        let src = Url::parse(image.src().to_str().unwrap()).unwrap();

        return Ok(Self::build_html_img(&src.to_string(), image.label().as_ref(), image.caption().as_ref(), id, img_classes, figure_style))
    }

    fn build_embed_remote_img(_image: &mut ImageResource, _id: Option<ResourceReference>, _img_classes: Vec<&str>, _figure_style: Option<String>, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<String, CompilationError> {
        unimplemented!("embed remote image will be added in a next version")
    }

    fn build_embed_local_img(image: &mut ImageResource, id: Option<ResourceReference>, img_classes: Vec<&str>, figure_style: Option<String>, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<String, CompilationError> {
        let base64_image = image.to_base64(compilation_configuration.compress_embed_image());

        if let Some(mt) = image.mime_type().as_ref() {

            return Ok(Self::build_html_img(&format!("data:{};base64,{}", mt, base64_image.unwrap()), image.label().as_ref(), image.caption().as_ref(), id, img_classes, figure_style));

        } else {
            if compilation_configuration.strict_image_src_check() {

                return Err(CompilationError::ResourceError(crate::resource::ResourceError::InvalidResourceVerbose(format!("image {:?} mime type not found", image.src()))));

            } else {

                log::warn!("{:?} will be compiled as local NOT embed image due to an error", image.src());

                return Ok(Self::build_not_embed_local_img(image, id, img_classes, figure_style, compilation_configuration, Arc::clone(&compilation_configuration_overlay)).unwrap());
            }
        }
    }

    fn build_not_embed_local_img(image: &mut ImageResource, id: Option<ResourceReference>, img_classes: Vec<&str>, figure_style: Option<String>, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<String, CompilationError> {
        let local_not_embed_src = fs::canonicalize(image.src()).unwrap();

        return Ok(Self::build_html_img(&local_not_embed_src.to_string_lossy().to_string(), image.label().as_ref(), image.caption().as_ref(), id.clone(), img_classes.clone(), figure_style.clone()));
    }


    fn build_img_from_compilation_configuration(image: &mut ImageResource, id: Option<ResourceReference>, img_classes: Vec<&str>, figure_style: Option<String>,  compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<String, CompilationError> {

        if RemoteResource::is_valid_remote_resource(image.src().to_str().unwrap()) {          // remote image (e.g. URL)

            if compilation_configuration.embed_remote_image() {

                return Self::build_embed_remote_img(image, id, img_classes, figure_style, compilation_configuration, Arc::clone(&compilation_configuration_overlay));

            } else {
                
                return Self::build_not_embed_remote_img(image, id, img_classes, figure_style, compilation_configuration, Arc::clone(&compilation_configuration_overlay));
            }

        } else {                // local image

            if image.src().exists() {

                if compilation_configuration.embed_local_image() {

                    return Self::build_embed_local_img(image, id, img_classes, figure_style, compilation_configuration, Arc::clone(&compilation_configuration_overlay));
                    
                } else {        // local not embed

                    return Ok(Self::build_not_embed_local_img(image, id, img_classes, figure_style, compilation_configuration, Arc::clone(&compilation_configuration_overlay)).unwrap());
                }


            } else if compilation_configuration.strict_image_src_check() {

                log::error!("{}", CompilationError::InvalidSource(String::from(image.src().to_string_lossy().to_string())));

                panic!("invalid src")

            } else {

                return Ok(Self::build_html_img(&image.src().to_string_lossy().to_string(), image.label().as_ref(), image.caption().as_ref(), id, img_classes, figure_style))       // create image tag of invalid image instead of panic
            }

        }

    }

    fn parse_image(search_pattern_regex: &Regex, content: &str, codex: &Codex,  compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        if !search_pattern_regex.is_match(content) {
            return Err(CompilationError::InvalidSource(format!("'{}' do not match using: {}", content, search_pattern_regex)))
        }

        let parsed_content = search_pattern_regex.replace_all(content, |captures: &Captures| {
            
            if let Some(label) = captures.get(1) {

                if let Some(src) = captures.get(3) {

                    let style: Option<String>;

                    if let Some(_style) = captures.get(4) {
                        style = Some(String::from(_style.as_str()));
                    } else {
                        style = None;
                    }

                    let parsed_label = Compiler::compile_str(label.as_str(), &OutputFormat::Html, codex, &compilation_configuration, Arc::clone(&compilation_configuration_overlay)).unwrap();

                    let binding = compilation_configuration_overlay.read().unwrap();
                    let document_name = binding.document_name().as_ref().unwrap();

                    if let Some(id) = captures.get(2) {

                        let id = ResourceReference::of_internal_from_without_sharp(id.as_str(), Some(document_name)).unwrap();

                        let mut image: ImageResource = ImageResource::new(PathBuf::from(src.as_str()), Some(parsed_label.content()), Some(label.as_str().to_string()))
                                                                        .elaborating_relative_path_as_dossier_assets(compilation_configuration.input_location())
                                                                        .inferring_mime_type()
                                                                        .unwrap();

                        return Self::build_img_from_compilation_configuration(&mut image, Some(id), vec!["image"], style, compilation_configuration, Arc::clone(&compilation_configuration_overlay)).unwrap();

                    } else {

                        let id = ResourceReference::of(label.as_str(), Some(document_name)).unwrap();

                        let image = ImageResource::new(PathBuf::from(src.as_str()), Some(parsed_label.content()), Some(label.as_str().to_string()))
                                                            .elaborating_relative_path_as_dossier_assets(compilation_configuration.input_location())
                                                            .inferring_mime_type();
                        
                        if let Err(err) = &image {
                            log::error!("error occurs during image '{}' loading: {}", src.as_str(), err.to_string());
                        }

                        let mut image = image.unwrap();

                        return Self::build_img_from_compilation_configuration(&mut image, Some(id), vec!["image"], style, compilation_configuration, Arc::clone(&compilation_configuration_overlay)).unwrap();
 
                    }
                }
            }

            unreachable!()
            
        }).to_string();
        
        Ok(CompilationResult::new_fixed(parsed_content))
    }

    fn parse_abridged_image(search_pattern_regex: &Regex, content: &str, _codex: &Codex,  compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        let binding = compilation_configuration_overlay.read().unwrap();
        let document_name = binding.document_name().as_ref().unwrap();

        if !search_pattern_regex.is_match(content) {
            return Err(CompilationError::InvalidSource(format!("'{}' do not match using: {}", content, search_pattern_regex)))
        }

        let parsed_content = search_pattern_regex.replace_all(content, |captures: &Captures| {
            
            let src = captures.get(1).unwrap();

            let id: Option<ResourceReference>;

            if let Some(_id) = captures.get(2) {
                id = Some(ResourceReference::of_internal_from_without_sharp(_id.as_str(), Some(document_name)).unwrap());
            } else {
                id = None;
            }

            let style: Option<String>;

            if let Some(_style) = captures.get(3) {
                style = Some(String::from(_style.as_str()));
            } else {
                style = None;
            }

            let mut image = ImageResource::new(PathBuf::from(src.as_str()), None, None)
                                                            .elaborating_relative_path_as_dossier_assets(compilation_configuration.input_location())
                                                            .inferring_mime_type()
                                                            .unwrap();

            return Self::build_img_from_compilation_configuration(&mut image, id, vec!["image", "abridged-image"], style, compilation_configuration, Arc::clone(&compilation_configuration_overlay)).unwrap();

        }).to_string();
        
        Ok(CompilationResult::new_fixed(parsed_content))
    }

    fn parse_multi_image(search_pattern_regex: &Regex, content: &str, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        let parsed_content = search_pattern_regex.replace_all(content, |captures: &Captures| {
            
            let justify_content: Option<String>;

            if let Some(jc) = captures.get(1) {
                justify_content = Some(String::from(jc.as_str()));
            } else {
                justify_content = None;
            }

            let raw_images = String::from(captures.get(2).unwrap().as_str());

            let images_container_style: String = format!("display: flex; justify-content: {};", justify_content.unwrap_or(String::from(DEFAULT_JUSTIFY_CONTENT)));
            let mut images_container = build_html::Container::new(build_html::ContainerType::Div)
                                                .with_attributes(vec![
                                                    ("style", images_container_style.as_str()),
                                                    ("class", "images-container")
                                                ]);

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

                let mut image_container = Container::new(build_html::ContainerType::Div)
                                                    .with_attributes(vec![
                                                        ("style", format!(r"align-self: {}", align_self).as_str()),
                                                        ("class", "image-container")
                                                    ]);

                for modifier in MULTI_IMAGE_PERMITTED_MODIFIER {
                    let parse_res = Self::parse_image_from_identifier(&modifier.identifier(), &Regex::new(&modifier.modifier_pattern()).unwrap(), raw_image_line, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay));

                    if let Ok(result) = parse_res {
                        image_container = image_container.with_raw(result.content());
                    }
                }

                images_container = images_container.with_container(image_container);
            }

            images_container.to_html_string()

        }).to_string();
        
        Ok(CompilationResult::new_fixed(parsed_content))
    }

    fn parse_image_from_identifier(image_modifier_identifier: &ModifierIdentifier, search_pattern_regex: &Regex, content: &str, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {
        

        if image_modifier_identifier.eq(&StandardParagraphModifier::Image.identifier()) {
            return Self::parse_image(search_pattern_regex, content, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay));
        }

        if image_modifier_identifier.eq(&StandardParagraphModifier::AbridgedImage.identifier()) {
            return Self::parse_abridged_image(search_pattern_regex, content, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay));        
        }

        if image_modifier_identifier.eq(&StandardParagraphModifier::MultiImage.identifier()) {
            return Self::parse_multi_image(search_pattern_regex, content, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay))
        }

        log::error!("'{}' is unsupported image modifier identifier", image_modifier_identifier);

        panic!("unsupported image modifier identifier");
    }
}

impl CompilationRule for HtmlImageRule {

    fn search_pattern(&self) -> &String {
        &self.search_pattern
    }

    fn standard_compile(&self, content: &str, _format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        Self::parse_image_from_identifier(&self.image_modifier_identifier, &self.search_pattern_regex, content, codex, compilation_configuration, Arc::clone(&compilation_configuration_overlay))
    }

    fn fast_compile(&self, content: &str, _format: &OutputFormat, _codex: &Codex, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {
        Ok(CompilationResult::new_fixed(format!(r#"<img alt="{}" />"#, content)))
    }
    
    fn search_pattern_regex(&self) -> &Regex {
        &self.search_pattern_regex
    }
}

#[cfg(test)]
mod test {

    use crate::codex::codex_configuration::CodexConfiguration;

    use super::*;

    #[test]
    fn parse_all_in_one() {

        let img_src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test-resources").join("wikipedia-logo.png");

        let image_rule = HtmlImageRule::new(StandardParagraphModifier::Image.identifier());

        let nmd_text = format!("\n\n![image1]({})\n\n", img_src.as_os_str().to_string_lossy());

        let codex = Codex::of_html(CodexConfiguration::default());

        let pc = CompilationConfiguration::default();

        let mut pco = CompilationConfigurationOverLay::default();

        pco.set_document_name(Some(String::from("test")));

        let parsed_content = image_rule.compile(nmd_text.as_str(), &OutputFormat::Html, &codex, &pc, Arc::new(RwLock::new(pco))).unwrap();
        
        assert!(parsed_content.parts().len() > 0)
    }
}