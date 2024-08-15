pub mod list_bullet_configuration_record;
pub mod compilation_configuration_overlay;

use std::{collections::HashMap, path::PathBuf};
use getset::{CopyGetters, Getters, MutGetters, Setters};
use crate::{bibliography::Bibliography, codex::{modifier::ModifiersBucket, CodexIdentifier}, resource::{bucket::Bucket, text_reference::TextReferenceMap}, theme::Theme};
use self::list_bullet_configuration_record::ListBulletConfigurationRecord;



#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompilableResourceType {
    Dossier,
    File,

    #[default]
    Unknown
}


/// Struct which contains all information about possible compilation options 
#[derive(Debug, Getters, CopyGetters, MutGetters, Setters, Clone)]
pub struct CompilationConfiguration {

    #[getset(get = "pub", set = "pub")]
    input_location: PathBuf,

    #[getset(get = "pub", set = "pub")]
    output_location: PathBuf,

    #[getset(get_copy = "pub", set = "pub")]
    embed_local_image: bool,

    #[getset(get_copy = "pub", set = "pub")]
    embed_remote_image: bool,
    
    #[getset(get_copy = "pub", set = "pub")]
    compress_embed_image: bool,

    #[getset(get_copy = "pub", set = "pub")]
    strict_image_src_check: bool,

    #[getset(get = "pub", set = "pub")]
    excluded_modifiers: ModifiersBucket,

    #[getset(get_copy = "pub", set = "pub")]
    parallelization: bool,

    #[getset(get = "pub", set = "pub")]
    list_bullets_configuration: Vec<ListBulletConfigurationRecord>,
    
    #[getset(get_copy = "pub", set = "pub")]
    strict_list_check: bool,

    #[getset(get_copy = "pub", set = "pub")]
    strict_focus_block_check: bool,

    #[getset(get = "pub", set = "pub")]
    references: TextReferenceMap,

    #[getset(get_copy = "pub", set = "pub")]
    fast_draft: bool,

    #[getset(get = "pub", set = "pub")]
    bibliography: Option<Bibliography>,

    #[getset(get = "pub", set = "pub")]
    theme: Theme,

    #[getset(get = "pub", set = "pub")]
    resource_type: CompilableResourceType,
}

impl CompilationConfiguration {

    pub fn new(input_location: PathBuf, output_location: PathBuf, embed_local_image: bool, embed_remote_image: bool, 
                compress_embed_image: bool, strict_image_src_check: bool, excluded_modifiers: ModifiersBucket, 
                parallelization: bool, list_bullets_configuration: Vec<ListBulletConfigurationRecord>, strict_list_check: bool, 
                strict_focus_block_check: bool, references: TextReferenceMap, fast_draft: bool, bibliography: Option<Bibliography>,
                theme: Theme, resource_type: CompilableResourceType) -> Self {

        Self {
            input_location,
            output_location,
            embed_local_image,
            embed_remote_image,
            compress_embed_image,
            strict_image_src_check,
            excluded_modifiers,
            parallelization,
            list_bullets_configuration,
            strict_list_check,
            strict_focus_block_check,
            references,
            fast_draft,
            bibliography,
            theme,
            resource_type,
        }
    }
}

impl Default for CompilationConfiguration {
    fn default() -> Self {
        Self {
            input_location: PathBuf::from("."),
            output_location: PathBuf::from("."),
            embed_local_image: true,
            embed_remote_image: false,
            compress_embed_image: false,
            strict_image_src_check: false,
            excluded_modifiers: Bucket::None,
            parallelization: false,
            list_bullets_configuration: list_bullet_configuration_record::default_bullets_configuration(),
            strict_list_check: false,
            strict_focus_block_check: false,
            references: HashMap::new(),
            fast_draft: false,
            bibliography: None,
            theme: Theme::default(),
            resource_type: CompilableResourceType::default(),
        }
    }
}