use oxipng::Options;
use crate::{compiler::compilation_result::CompilationResult, resource::{image_resource::ImageResource, source::Source, ResourceError}};
use super::nmd_unique_identifier::NmdUniqueIdentifier;


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


pub fn compile_image_resource_in_html(image: &ImageResource, img_classes: Vec<&str>, nuid: Option<&NmdUniqueIdentifier>) -> Result<CompilationResult, ResourceError> {
    let id_attr: String;

    if let Some(id) = image.id() {
        id_attr = format!(r#"id="{}""#, id.build_without_internal_sharp());
    } else {
        id_attr = String::new();
    }

    // let html_alt: String;
    let html_caption: String;

    // if let Some(a) = alt {
    //     html_alt = format!(r#"alt="{}""#, a);
    // } else {
    //     html_alt = String::new();
    // }

    if let Some(c) = image.caption() {

        html_caption = format!(r#"<figcaption class="image-caption">{}</figcaption>"#, c);
    } else {
        html_caption = String::new();
    }

    let style_attr: String;

    if let Some(style) = image.style() {
        style_attr = format!(r#"style="{}""#, style);
    } else {
        style_attr = String::new();
    }

    let nuid_attr: String;

    if let Some(nuid) = nuid {
        nuid_attr = format!(r#"data-nuid="{}""#, nuid);
    } else {
        nuid_attr = String::new();
    }

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

    Ok(format!(r#"<figure class="figure" {} {}>
            <img src="{}" class="{}" {} />
            {}
        </figure>"#, id_attr, nuid_attr, src, img_classes.join(" "), style_attr, html_caption
    ))
} 