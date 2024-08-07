use std::collections::HashMap;
use ahash::RandomState;
use crate::dossier::Document;
use super::file_utility;


/// `NmdUniqueIdentifier` is a unique identifier in a NMD compilation.
pub type NmdUniqueIdentifier = String;

const HASHER_SEED: usize = 42;

pub fn assign_nuid_to_document_paragraphs(document: &mut Document) {

    let hasher = RandomState::with_seed(HASHER_SEED);
    let mut nuid_map: HashMap<u64, usize> = HashMap::new();

    let prefix = file_utility::build_output_file_name(document.name(), None);

    let mut calc_nuid = |s: &String| {

        let h = hasher.hash_one(s);

        let mut n = *nuid_map.get(&h).unwrap_or(&0);

        n += 1;

        nuid_map.insert(h, n);

        let nuid = format!("{}-{}-{}", prefix, h, n);

        nuid
    };
    
    document.preamble_mut().iter_mut().for_each(|p| {
        p.set_nuid(Some(calc_nuid(p.content())));
    });

    document.chapters_mut().iter_mut().for_each(|chapter| {
        chapter.paragraphs_mut().iter_mut().for_each(|p| {
            p.set_nuid(Some(calc_nuid(p.content())));
        });

        let title = chapter.heading().title().clone();
        chapter.heading_mut().set_nuid(Some(calc_nuid(&title)));
    });
}


#[cfg(test)]
mod test {
    use ahash::RandomState;

    use super::HASHER_SEED;

    #[test]
    fn deterministic_hashing() {

        let test_string = "this is a test string";

        let hasher = RandomState::with_seed(HASHER_SEED);
        let h1 = hasher.hash_one(test_string);

        let hasher = RandomState::with_seed(HASHER_SEED);
        let h2 = hasher.hash_one(test_string);

        assert_eq!(h1, h2);
    }
}