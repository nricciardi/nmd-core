use std::str::FromStr;

use build_html::{HtmlPage, HtmlContainer, Html, Container};
use getset::{Getters, Setters};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

use crate::{artifact::Artifact, bibliography::Bibliography, compiler::compilation_result_accessor::CompilationResultAccessor, dossier::{document::chapter::chapter_tag::ChapterTagKey, Document, Dossier}, resource::{dynamic_resource::DynamicResource, Resource, ResourceError}, table_of_contents::TableOfContents, theme::Theme};

use super::{Assembler, AssemblerError, assembler_configuration::AssemblerConfiguration};


#[derive(Debug, Getters, Setters)]
pub struct HtmlAssembler {
}

impl HtmlAssembler {

    fn apply_standard_remote_addons(mut page: HtmlPage, theme: &Theme) -> HtmlPage {

        // add code block js/css
        match theme {
            Theme::Light | Theme::HighContrast | Theme::None => {
                page = page
                    .with_script_link_attr("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js", [
                        ("crossorigin", "anonymous"),
                    ])
                    .with_script_link_attr("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/plugins/autoloader/prism-autoloader.min.js", [
                        ("crossorigin", "anonymous"),
                    ])
                    .with_head_link("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism.css", "stylesheet")
                    .with_head_link("https://emoji-css.afeld.me/emoji.css", "stylesheet");
                
            },
            Theme::Dark => {
                page = page
                    .with_script_link_attr("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js", [
                        ("crossorigin", "anonymous"),
                    ])
                    .with_script_link_attr("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/plugins/autoloader/prism-autoloader.min.js", [
                        ("crossorigin", "anonymous"),
                    ])
                    .with_head_link("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-okaidia.css", "stylesheet")
                    .with_head_link("https://emoji-css.afeld.me/emoji.css", "stylesheet");
            },
            Theme::Scientific => {
                page = page
                    .with_script_link_attr("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js", [
                        ("crossorigin", "anonymous"),
                    ])
                    .with_script_link_attr("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/plugins/autoloader/prism-autoloader.min.js", [
                        ("crossorigin", "anonymous"),
                    ])
                    .with_head_link("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-coy.css", "stylesheet")
                    .with_head_link("https://emoji-css.afeld.me/emoji.css", "stylesheet");
            },
            Theme::Vintage => {
                page = page
                    .with_script_link_attr("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/components/prism-core.min.js", [
                        ("crossorigin", "anonymous"),
                    ])
                    .with_script_link_attr("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/plugins/autoloader/prism-autoloader.min.js", [
                        ("crossorigin", "anonymous"),
                    ])
                    .with_head_link("https://cdnjs.cloudflare.com/ajax/libs/prism/1.29.0/themes/prism-solarizedlight.css", "stylesheet")
                    .with_head_link("https://emoji-css.afeld.me/emoji.css", "stylesheet");
            }
        };

        // add math block js/css
        page = page
                .with_head_link_attr("https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.css", "stylesheet", [
                    ("integrity", "sha384-n8MVd4RsNIU0tAv4ct0nTaAbDJwPJzDEaqSD1odI+WdtXRGWt2kTvGFasHpSy3SV"),
                    ("crossorigin", "anonymous")
                ])
                .with_script_link_attr("https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/katex.min.js", [
                    ("integrity", "sha384-XjKyOOlGwcjNTAIQHIpgOno0Hl1YQqzUOEleOLALmuqehneUG+vnGctmUb0ZY0l8"),
                    ("crossorigin", "anonymous")
                ])
                .with_script_link_attr("https://cdn.jsdelivr.net/npm/katex@0.16.9/dist/contrib/auto-render.min.js", [
                    ("integrity", "sha384-+VBxd3r6XgURycqtZ117nYw44OOcIax56Z4dCRWbxyPt0Koah1uHoK0o4+/RRE05"),
                    ("crossorigin", "anonymous")
                ]);

        page.add_script_literal(r#"
                document.addEventListener("DOMContentLoaded", function() {
                    renderMathInElement(document.body, {
                        
                        delimiters: [
                            {left: '$$', right: '$$', display: true},
                            {left: '$', right: '$', display: false},
                        ],
                        
                        throwOnError : false
                    });
                });"#);

        page
    }

    fn apply_standard_local_addons(mut page: HtmlPage, theme: &Theme) -> HtmlPage {

        page.add_style(include_str!("html_assembler/emoji/emoji.min.css"));
        
        page.add_style(include_str!("html_assembler/math_block/katex.css"));
        page.add_style(include_str!("html_assembler/math_block/katex-fonts.css"));
        page.add_script_literal(include_str!("html_assembler/math_block/katex.min.js"));
        page.add_script_literal(include_str!("html_assembler/math_block/auto-render.min.js"));
        page.add_script_literal(r#"window.onload = function() {
            renderMathInElement(document.body, {
                delimiters: [
                    {left: '$$', right: '$$', display: true},
                    {left: '$', right: '$', display: false},
                ],
                throwOnError : false
                });
        }"#);

        // add code block js/css                        
        match theme {
            Theme::Light | Theme::HighContrast | Theme::None => {
                page.add_style(include_str!("html_assembler/code_block/light_theme/prismjs.css"));
                page.add_script_literal(include_str!("html_assembler/code_block/light_theme/prismjs.js"));
            },
            Theme::Dark => {
                page.add_style(include_str!("html_assembler/code_block/dark_theme/prismjs.css"));
                page.add_script_literal(include_str!("html_assembler/code_block/dark_theme/prismjs.js"));
            },
            Theme::Scientific => {
                page.add_style(include_str!("html_assembler/code_block/scientific_theme/prismjs.css"));
                page.add_script_literal(include_str!("html_assembler/code_block/scientific_theme/prismjs.js"));
            },
            Theme::Vintage => {
                page.add_style(include_str!("html_assembler/code_block/vintage_theme/prismjs.css"));
                page.add_script_literal(include_str!("html_assembler/code_block/vintage_theme/prismjs.js"));
            },
        };

        page
    }

    fn apply_theme_style(mut page: HtmlPage, theme: &Theme) -> HtmlPage {

        match theme {
            Theme::Light => page.add_style(include_str!("html_assembler/default_style/light_theme.css")),
            Theme::Dark => page.add_style(include_str!("html_assembler/default_style/dark_theme.css")),
            Theme::Scientific => page.add_style(include_str!("html_assembler/default_style/scientific_theme.css")),
            Theme::Vintage => page.add_style(include_str!("html_assembler/default_style/vintage_theme.css")),
            Theme::HighContrast => {
                page.add_style(include_str!("html_assembler/default_style/light_theme.css"));
                page.add_style(include_str!("html_assembler/default_style/high_contrast_theme.css"));
            },
            Theme::None => ()       // nothing,
        }

        page
    }

    fn apply_styles(mut page: HtmlPage, styles_references: &Vec<String>) -> Result<HtmlPage, AssemblerError> {
        for ref style_ref in styles_references {

            log::info!("appending style (reference): {:?}", style_ref);

            let resource = DynamicResource::from_str(style_ref)?;

            match resource {
                DynamicResource::DiskResource(disk_resource) => page.add_style(disk_resource.read()?),
                DynamicResource::ImageResource(_) => return Err(AssemblerError::ResourceError(ResourceError::InvalidResourceVerbose("image cannot be an addons".to_string()))),
                DynamicResource::RemoteResource(remote_resource) => {
                    page = page.with_script_link_attr(remote_resource.location().to_string(), [
                        ("crossorigin", "anonymous"),
                    ])
                },
            }
        }

        Ok(page)
    }

    fn apply_check_preview_update_script(mut page: HtmlPage) -> Result<HtmlPage, AssemblerError> {

        page.add_script_literal(include_str!("html_assembler/check_preview_updates.js"));

        Ok(page)
    }

    fn create_default_html_page(page_title: &String, styles_references: &Vec<String>, theme: &Theme, use_remote_addons: bool) -> Result<HtmlPage, AssemblerError> {

        let mut page = HtmlPage::new()
                                    .with_title(page_title)
                                    .with_meta(vec![("charset", "utf-8")]);

        if use_remote_addons {
        page = Self::apply_standard_remote_addons(page, theme);

        } else {
        page = Self::apply_standard_local_addons(page, theme);
        }

        page = Self::apply_theme_style(page, theme);

        page = Self::apply_styles(page, &styles_references)?;

        Ok(page)
    }
}

impl Assembler for HtmlAssembler {

    fn assemble_dossier(dossier: &Dossier, configuration: &AssemblerConfiguration) -> Result<Artifact, AssemblerError> {
                        
        if dossier.documents().is_empty() {
            return Err(AssemblerError::TooFewElements("there are no documents".to_string()))
        }

        let mut styles_references = dossier.configuration().style().styles_references();
        log::info!("appending {} custom styles", styles_references.len());

        let mut other_styles = configuration.styles_raw_path().clone();
        styles_references.append(&mut other_styles);

        let mut page = Self::create_default_html_page(dossier.name(), &styles_references, configuration.theme(), configuration.use_remote_addons())?;
        
        if let Some(toc) = dossier.table_of_contents() {
            if let Some(compiled_toc) = toc.compilation_result() {
                page.add_raw(compiled_toc.content());
            }
        }

        if configuration.parallelization() {

            let mut assembled_documents: Vec<Result<Artifact, AssemblerError>> = Vec::new();

            dossier.documents().par_iter().map(|document| {
                Self::assemble_document(document, configuration)
            }).collect_into_vec(&mut assembled_documents);

            for assembled_document in assembled_documents {
                let section = Container::new(build_html::ContainerType::Section)
                                                .with_attributes(vec![
                                                    ("class", "document")
                                                ])
                                                .with_raw(assembled_document?);
    
                page.add_container(section);
            }

        } else {

            for document in dossier.documents() {
                let section = Container::new(build_html::ContainerType::Section)
                                                .with_attributes(vec![
                                                    ("class", "document")
                                                ])
                                                .with_raw(Self::assemble_document(document, configuration)?);
    
                page.add_container(section);
            }
        }

        if let Some(bib) = dossier.bibliography() {
            if let Some(compiled_bib) = bib.compilation_result() {
                page.add_raw(compiled_bib.content());
            }
        }

        let artifact = Artifact::new(page.to_html_string());

        Ok(artifact)
    }
    
    fn assemble_document(document: &Document, _configuration: &AssemblerConfiguration) -> Result<Artifact, AssemblerError> {
        let mut result = String::new();

        for paragraph in document.preamble() {

            if let Some(compiled_content) = paragraph.compilation_result().as_ref() {

                result.push_str(&compiled_content.content());

            } else {
                return Err(AssemblerError::CompiledContentNotFound)
            }
        }

        for chapter in document.chapters() {

            let mut div_chapter = Container::new(build_html::ContainerType::Div);
            let mut style = String::new();

            for tag in chapter.tags() {

                match tag.key() {
                    ChapterTagKey::Id => {
                        div_chapter = div_chapter.with_attributes(vec![("id", tag.value().as_ref().unwrap().as_str())])
                    }
                    ChapterTagKey::Style => {
                        style.push_str(format!("{};", tag.value().as_ref().unwrap().as_str()).as_str())
                    },
                    ChapterTagKey::StyleClass => {
                        div_chapter = div_chapter.with_attributes(vec![("class", tag.value().as_ref().unwrap().as_str())])
                    },

                    _ => {
                        log::warn!("chapter tag key not supported yet")
                    }
                }
            }

            div_chapter = div_chapter.with_attributes(vec![("style", style.as_str())]);
            let mut div_chapter_content = String::new();

            if let Some(compiled_content) = chapter.heading().compilation_result().as_ref() {

                div_chapter_content.push_str(&compiled_content.content());

            } else {
                return Err(AssemblerError::CompiledContentNotFound)
            }

            for paragraph in chapter.paragraphs() {
                if let Some(compiled_content) = paragraph.compilation_result().as_ref() {

                    let compiled_content = compiled_content.content();

                    if compiled_content.is_empty() {
                        continue;
                    }

                    div_chapter_content.push_str(&compiled_content);
    
                } else {
                    return Err(AssemblerError::CompiledContentNotFound)
                }
            }

            result.push_str(div_chapter.with_raw(div_chapter_content).to_html_string().as_str());
        }

        Ok(Artifact::new(result))
    }

    fn assemble_document_standalone(document: &Document, page_title: &String, styles_references: Option<&Vec<String>>, toc: Option<&TableOfContents>, bibliography: Option<&Bibliography>, configuration: &AssemblerConfiguration) -> Result<Artifact, AssemblerError> {
        let mut page = Self::create_default_html_page(page_title, styles_references.unwrap_or(&Vec::new()), configuration.theme(), configuration.use_remote_addons())?;

        if let Some(toc) = toc {
            if let Some(compiled_toc) = toc.compilation_result() {
                page.add_raw(compiled_toc.content());
            }
        }

        page.add_raw(Into::<String>::into(Self::assemble_document(document, configuration)?));

        if let Some(bib) = bibliography {
            if let Some(compiled_bib) = bib.compilation_result() {
                page.add_raw(compiled_bib.content());
            }
        }

        Ok(Artifact::new(page.to_html_string()))
    }
}

// #[cfg(test)]
// mod test {

//     use std::{path::PathBuf, sync::Arc};

//     use crate::compiler::{codex::Codex, dossier::dossier_configuration::DossierConfiguration, parser::parsing_rule::parsing_configuration::ParsingConfiguration};

//     use super::*;

//     #[test]
//     fn assemble() {

//         let codex = Arc::new(Codex::of_html(CodexConfiguration::default()));

//         let project_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
//         let dossier_dir = "nmd-test-dossier-1";
//         let nmd_file = project_directory.join("test-resources").join(dossier_dir).join("d1.nmd");

//         assert!(nmd_file.is_file());

//         let mut dossier_configuration = DossierConfiguration::default();
//         dossier_configuration.set_raw_documents_paths(vec![nmd_file.to_string_lossy().to_string()]);

//         let mut dossier = Dossier::load(Arc::clone(&codex), &dossier_configuration).unwrap();

//         dossier.parse(Arc::clone(&codex), Arc::new(ParsingConfiguration::default())).unwrap();

//         let assembler = HtmlAssembler::new(AssemblerConfiguration::default());

//         let _ = assembler.assemble(codex.into(), *dossier).unwrap();
//     }
// }