use oxipng::Options;
use crate::{codex::{modifier::ModifiersBucket, Codex}, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, compilation::{compilable::Compilable, compilation_configuration::{compilation_configuration_overlay::CompilationConfigurationOverLay, CompilationConfiguration}, compilation_error::CompilationError, compilation_outcome::CompilationOutcome}, output_format::OutputFormat, resource::{image_resource::ImageResource, source::Source, ResourceError}};
use super::{nmd_unique_identifier::NmdUniqueIdentifier, text_utility};


pub fn set_image_base64_embed_src(image: &mut ImageResource, compression: bool) -> Result<(), ResourceError> {

    let src = image.src().clone();

    let src = src.try_into_bytes()?;

    if let Source::Bytes { mut bytes } = src {

        if compression {
    
            let original_log_level = log::max_level();
            log::set_max_level(log::LevelFilter::Warn);
    
            let options = Options::max_compression();
    
            let optimized_png = oxipng::optimize_from_memory(&bytes, &options);
    
            log::set_max_level(original_log_level);
    
            match optimized_png {
                Ok(image) => bytes = image,
                Err(err) => return Err(ResourceError::ElaborationError(format!("image compression error: {}", err)))
            }
    
        }

        let src = Source::Bytes { bytes };

        image.set_src(src.try_into_base64()?);

        return Ok(())
    }

    unreachable!("'try_into_bytes' must return bytes type")
}


pub fn compile_image_resource_in_html(image: &ImageResource, img_classes: Vec<&str>, nuid: Option<&NmdUniqueIdentifier>, codex: &Codex, compilation_configuration: &CompilationConfiguration, compilation_configuration_overlay: CompilationConfigurationOverLay) -> Result<CompilationOutcome, CompilationError> {
    
    let mut outcome = String::new();
    
    let id_attr: String;

    if let Some(id) = image.id() {
        id_attr = format!(r#"id="{}""#, id.build_without_internal_sharp());
    } else {
        id_attr = String::new();
    }

    let styles: String;
    let classes: String;

    if let Some(style) = image.style() {

        let (s, c) = text_utility::split_styles_and_classes(style);

        styles = s.unwrap_or(String::new());
        classes = c.unwrap_or(String::new());
    
    } else {

        styles = String::new();
        classes = String::new();
    }

    outcome.push_str(&format!(r#"<figure class="figure {}" style="{}" {} {}>"#, classes, styles, id_attr, text_utility::html_nuid_tag_or_nothing(nuid)));

    let src: String = match image.src() {
        Source::Remote { url: _ } | Source::Local { path: _ } => image.src().to_string(),
        Source::Base64String { base64 } => {
            if image.mime_type().is_none() {

                panic!("base64 image without mime type")
            }

            format!("data:{};base64,{}", image.mime_type().as_ref().unwrap(), base64)
        },
        Source::Bytes { bytes: _ } => {

            let base64 = image.src().try_to_base64()?;

            if image.mime_type().is_none() {

                panic!("base64 image without mime type")
            }

            format!("data:{};base64,{}", image.mime_type().as_ref().unwrap(), base64)
        },
    };

    outcome.push_str(&format!(r#"<img src="{}" class="{}" />"#, src, img_classes.join(" ")));


    if let Some(caption) = image.caption() {

        outcome.push_str(r#"<figcaption class="image-caption">"#);

        let mut compilable_text = CompilableText::from(CompilableTextPart::new_compilable(caption.clone(), ModifiersBucket::None));

        outcome.push_str(&compilable_text.compile(&OutputFormat::Html, codex, compilation_configuration, compilation_configuration_overlay.clone())?.content());
        
        outcome.push_str(r#"</figcaption>"#);
    }

    outcome.push_str("</figure>");

    Ok(CompilationOutcome::from(outcome))
} 