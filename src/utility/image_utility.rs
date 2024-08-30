use oxipng::Options;
use crate::{codex::modifier::ModifiersBucket, compilable_text::{compilable_text_part::CompilableTextPart, CompilableText}, resource::{image_resource::ImageResource, source::Source, ResourceError}};
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


pub fn compile_image_resource_in_html(image: &ImageResource, img_classes: Vec<&str>, nuid: Option<&NmdUniqueIdentifier>) -> Result<CompilableText, ResourceError> {
    
    let mut compilation_result = CompilableText::new_empty();
    
    let id_attr: String;

    if let Some(id) = image.id() {
        id_attr = format!(r#"id="{}""#, id.build_without_internal_sharp());
    } else {
        id_attr = String::new();
    }

    let style_attr: String;

    if let Some(style) = image.style() {
        style_attr = format!(r#"style="{}""#, style);
    } else {
        style_attr = String::new();
    }

    compilation_result.parts_mut().push(CompilableTextPart::new_fixed(format!(r#"<figure class="figure" {} {} {}>"#, id_attr, text_utility::html_nuid_tag_or_nothing(nuid), style_attr)));

    // let html_alt: String;

    // if let Some(a) = alt {
    //     html_alt = format!(r#"alt="{}""#, a);
    // } else {
    //     html_alt = String::new();
    // }

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

    compilation_result.parts_mut().push(CompilableTextPart::new_fixed(format!(r#"<img src="{}" class="{}" />"#, src, img_classes.join(" "))));


    if let Some(caption) = image.caption() {

        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"<figcaption class="image-caption">"#)));
        compilation_result.parts_mut().push(CompilableTextPart::new_compilable(caption.clone(), ModifiersBucket::None));
        compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from(r#"</figcaption>"#)));
    }

    compilation_result.parts_mut().push(CompilableTextPart::new_fixed(String::from("</figure>")));

    Ok(compilation_result)
} 