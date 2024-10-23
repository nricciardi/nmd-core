use build_html::{Container, Html, HtmlContainer};
use getset::{Getters, Setters};
use crate::{codex::Codex, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compiled_text_accessor::CompiledTextAccessor, self_compile::SelfCompile, Compiler}, dossier::document::chapter::paragraph::Paragraph, output_format::OutputFormat, resource::{image_resource::ImageResource, source::Source, ResourceError}, utility::{image_utility, nmd_unique_identifier::NmdUniqueIdentifier}};


const SINGLE_IMAGE_CLASSES: [&str; 1] = ["image"];
const ABRIDGED_IMAGE_CLASSES: [&str; 2] = ["image", "abridged-image"];

#[derive(Debug, Getters, Setters)]
pub struct MultiImage {
    
    #[getset(get = "pub", set = "pub")]
    pub alignment: String,

    /// (image resource, image alignment)
    #[getset(get = "pub", set = "pub")]
    pub images: Vec<(ImageParagraphContent, String)>, 
}


#[derive(Debug)]
pub enum ImageParagraphContent {
    SingleImage(ImageResource),
    AbridgedImage(ImageResource),
    MultiImage(MultiImage)
}


#[derive(Debug, Getters, Setters)]
pub struct ImageParagraph {

    #[getset(set = "pub")]
    nuid: Option<NmdUniqueIdentifier>,

    #[getset(set = "pub")]
    raw_content: String,
    
    content: ImageParagraphContent,

    #[getset(set = "pub")]
    compiled_content: Option<CompilableText>,

}


impl ImageParagraph {

    pub fn new(raw_content: String, content: ImageParagraphContent) -> Self {
        Self {
            raw_content,
            nuid: None,
            content,
            compiled_content: None
        }
    }

    fn html_standard_compile_single_or_abridged_image(content: &mut ImageParagraphContent, nuid: Option<&NmdUniqueIdentifier>, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<String, CompilationError> {

        let img_classes = match &content {
            ImageParagraphContent::SingleImage(_) => SINGLE_IMAGE_CLASSES.to_vec(),
            ImageParagraphContent::AbridgedImage(_) => ABRIDGED_IMAGE_CLASSES.to_vec(),
            ImageParagraphContent::MultiImage(_) => panic!("content {:#?} must be a single image", content),
        };

        match content {
            ImageParagraphContent::SingleImage(image) | ImageParagraphContent::AbridgedImage(image) => {

                match image.src() {
                    Source::Remote { url: _ } => {

                        if compilation_configuration.embed_remote_image() {
                            image_utility::set_image_base64_embed_src(image, compilation_configuration.compress_embed_image())?;
                        }

                        let mut compilable_text = image_utility::compile_image_resource_in_html(image, img_classes, nuid)?;

                        Compiler::compile_compilable_text(&mut compilable_text, &OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
                        
                        return Ok(compilable_text.content())
                    },
                    Source::Local { path } => {

                        if compilation_configuration.embed_local_image() {
                        
                            image_utility::set_image_base64_embed_src(image, compilation_configuration.compress_embed_image())?;
                        
                        } else {

                            let path = match std::fs::canonicalize(path) {
                                Ok(p) => p,
                                Err(_) => return Err(CompilationError::ResourceError(ResourceError::ResourceNotFound(path.to_string_lossy().to_string()))),
                            };

                            image.set_src(Source::Local { path });
                        }

                        let mut compilable_text = image_utility::compile_image_resource_in_html(image, img_classes, nuid)?;

                        Compiler::compile_compilable_text(&mut compilable_text, &OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

                        return Ok(compilable_text.content())
                    },
                    Source::Base64String { base64: _ } => {

                        let mut compilable_text = image_utility::compile_image_resource_in_html(image, img_classes, nuid)?;

                        Compiler::compile_compilable_text(&mut compilable_text, &OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?;

                        return Ok(compilable_text.content())
                    },
                    Source::Bytes { bytes: _ } => todo!(),
                }

            },

            ImageParagraphContent::MultiImage(_) => unreachable!(),
        }
        
    }

    fn html_standard_compile_multi_image(multi_image: &mut MultiImage, nuid: Option<&NmdUniqueIdentifier>, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<String, CompilationError> {

        let images_container_style: String = format!("display: flex; justify-content: {};", multi_image.alignment);
        let mut images_container = build_html::Container::new(build_html::ContainerType::Div)
                                            .with_attributes(vec![
                                                ("style", images_container_style.as_str()),
                                                ("class", "images-container")
                                            ]);

        for (content, alignment) in &mut multi_image.images {

            let image_html_tag = Self::html_standard_compile_single_or_abridged_image(content, nuid, codex, compilation_configuration, compilation_configuration_overlay.clone())?;
            
            let image_container = Container::new(build_html::ContainerType::Div)
                                                .with_attributes(vec![
                                                    ("style", format!(r"align-self: {}", alignment).as_str()),
                                                    ("class", "image-container")
                                                ])
                                                .with_raw( image_html_tag);

            images_container.add_container(image_container);
        }

        Ok(images_container.to_html_string())
    }

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        self.compiled_content = Some(match self.content {
            ImageParagraphContent::SingleImage(_) | ImageParagraphContent::AbridgedImage(_) => {
                CompilableText::from(CompilableTextPart::new_fixed(Self::html_standard_compile_single_or_abridged_image(&mut self.content, self.nuid.as_ref(), codex, compilation_configuration, compilation_configuration_overlay.clone())?))
            },
            ImageParagraphContent::MultiImage(ref mut multi_image) => {
                CompilableText::from(CompilableTextPart::new_fixed(Self::html_standard_compile_multi_image(multi_image, self.nuid.as_ref(), codex, compilation_configuration, compilation_configuration_overlay.clone())?))
            },
        });

        Ok(())
    }

    fn html_fast_compile(&mut self, _codex: &Codex, _compilation_configuration: &CompilationConfiguration, _compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        

        match &self.content {
            ImageParagraphContent::SingleImage(image) => {
                self.compiled_content = Some(image_utility::compile_image_resource_in_html(image, SINGLE_IMAGE_CLASSES.to_vec(), self.nuid.as_ref())?);
            },
            ImageParagraphContent::AbridgedImage(image) => {
                self.compiled_content = Some(image_utility::compile_image_resource_in_html(image, ABRIDGED_IMAGE_CLASSES.to_vec(), self.nuid.as_ref())?);
            }
            ImageParagraphContent::MultiImage(multi_image) => {
                self.compiled_content = Some(CompilableText::from(CompilableTextPart::new_fixed(format!(r#"<img alt="multi-image paragraph with {} image(s)" />"#, multi_image.images.len()))))
            },
        };
        
        Ok(())
    }
}


impl SelfCompile for ImageParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay.clone()),
        }
    }

    fn fast_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<(), CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_fast_compile(codex, compilation_configuration, compilation_configuration_overlay.clone()),
        }    
    }
}


impl CompiledTextAccessor for ImageParagraph {
    fn compiled_text(&self) -> Option<&CompilableText> {
        self.compiled_content.as_ref()
    }
}

impl Paragraph for ImageParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> Option<&NmdUniqueIdentifier> {
        self.nuid.as_ref()
    }
    
    fn set_raw_content(&mut self, raw_content: String) {
        self.raw_content = raw_content;
    }
    
    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.nuid = nuid;
    }
}
