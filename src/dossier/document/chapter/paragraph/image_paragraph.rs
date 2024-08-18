use std::sync::{Arc, RwLock};
use getset::{Getters, Setters};
use once_cell::sync::Lazy;
use regex::Regex;
use crate::{codex::{modifier::standard_paragraph_modifier::StandardParagraphModifier, Codex}, compiler::{compilable::{Compilable, GenericCompilable}, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, list_bullet_configuration_record::{self, ListBulletConfigurationRecord}, CompilationConfiguration}, compilation_error::CompilationError, compilation_result::CompilationResult, compilation_result_accessor::CompilationResultAccessor, compilation_rule::{constants::{ESCAPE_HTML, SPACE_TAB_EQUIVALENCE}, CompilationRule}, self_compile::SelfCompile, Compiler}, dossier::document::chapter::paragraph::ParagraphTrait, output_format::OutputFormat, resource::{image_resource::ImageResource, source::Source}, utility::{image_utility, nmd_unique_identifier::NmdUniqueIdentifier, text_utility}};


#[derive(Debug)]
pub struct MultiImage {
    
    pub alignment: String,

    /// (image resource, image alignment)
    pub images: Vec<(ImageResource, String)>, 
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
    compiled_content: Option<CompilationResult>,

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

    fn html_standard_compile_single_or_abridged_image(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        match &mut self.content {
            ImageParagraphContent::SingleImage(image) | ImageParagraphContent::AbridgedImage(image) => {

                match image.src() {
                    Source::Remote { url } => {

                        if compilation_configuration.embed_remote_image() {
                            image_utility::set_image_base64_embed_src(image, compilation_configuration.compress_embed_image())?;
                        }

                        return image_utility::compile_image_resource_in_html(image, img_classes, nuid)


                    },
                    Source::Local { path } => todo!(),
                    Source::Base64String { base64 } => todo!(),
                    Source::Bytes { bytes } => todo!(),
                }

            },

            ImageParagraphContent::MultiImage(_) => panic!("content {:#?} must be a single image", self.content),
        }
        
    }

    fn html_standard_compile_single_or_abridged_image(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<CompilationResult, CompilationError> {

        if let ImageParagraphContent::MultiImage(ref multi_image) = self.content {

            

        }

        panic!("content {:#?} must be a single image", self.content)
    }

    fn html_standard_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        self.compiled_content = Some(match self.content {
            ImageParagraphContent::SingleImage(_) | ImageParagraphContent::AbridgedImage(_) => self.html_standard_compile_single_or_abridged_image(codex, compilation_configuration, compilation_configuration_overlay.clone())?,
            ImageParagraphContent::MultiImage(_) => todo!(),    // TODO
        });

        // match &mut self.content {
        //     ImageParagraphContent::SingleImage(image) | ImageParagraphContent::AbridgedImage(image) => {


                

        //         CompilationResult::new_fixed(image_utility::build_html_img_tag(image, vec![], self.nuid.as_ref())?);
        //     },
        //     ImageParagraphContent::MultiImage(multi_image) => {
        //         todo!()     
        //     },
        // };
        
        Ok(())
    }

    fn html_fast_compile(&mut self, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        

        match &self.content {
            ImageParagraphContent::SingleImage(image) | ImageParagraphContent::AbridgedImage(image) => {
                self.compiled_content = Some(CompilationResult::new_fixed(image_utility::compile_image_resource_in_html(image, vec![], self.nuid.as_ref())?));
            },
            ImageParagraphContent::MultiImage(multi_image) => {
                todo!()     // TODO
            },
        };
        
        Ok(())
    }
}


impl SelfCompile for ImageParagraph {
    fn standard_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_standard_compile(codex, compilation_configuration, compilation_configuration_overlay.clone()),
        }
    }

    fn fast_compile(&mut self, format: &OutputFormat, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: Arc<RwLock<CompilationConfigurationOverLay>>) -> Result<(), CompilationError> {
        
        match format {
            OutputFormat::Html => self.html_fast_compile(codex, compilation_configuration, compilation_configuration_overlay.clone()),
        }    
    }
}


impl CompilationResultAccessor for ImageParagraph {
    fn compilation_result(&self) -> &Option<CompilationResult> {
        &self.compiled_content
    }
}

impl ParagraphTrait for ImageParagraph {
    fn raw_content(&self) -> &String {
        &self.raw_content
    }

    fn nuid(&self) -> &Option<NmdUniqueIdentifier> {
        &self.nuid
    }
    
    fn set_raw_content(&mut self, raw_content: String) {
        self.raw_content = raw_content;
    }
    
    fn set_nuid(&mut self, nuid: Option<NmdUniqueIdentifier>) {
        self.nuid = nuid;
    }
}
