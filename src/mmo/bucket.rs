use std::hash::Hash;

use serde::Serialize;

use super::HashSet;


#[derive(Debug, Clone, Default, Serialize)]
pub enum Bucket<T> {
    All,
    Set(HashSet<T>),

    #[default]
    None
}

impl<T: Eq + Hash> Bucket<T> {
    pub fn contains(&self, item: &T) -> bool {
        match self {
            Self::All => true,
            Self::Set(set) => set.contains(item),
            Self::None => false,
        }
    }

    pub fn insert(mut self, item: T) -> Self {
        match self {
            Self::All => Self::All,
            Self::Set(ref mut set) => {
                set.insert(item);

                self
            },
            Self::None => Self::Set(HashSet::from([item])),
        }
    }    
}

impl<T: Eq + Hash + Clone> Bucket<T> {
    pub fn extend(mut self, b: &Bucket<T>) -> Self {
        match b {
            Self::All => Self::All,
            Self::Set(set) => {
                for item in set {
                    self = self.insert(item.clone());
                }

                self
            }
            Self::None => self,
        }
    }
}

impl<T: Eq + Hash> From<T> for Bucket<T> {
    fn from(value: T) -> Self {
        
        Self::Set(HashSet::from([value]))
    }
}