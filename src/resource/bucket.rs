use std::ops::Add;


#[derive(Debug, PartialEq, Clone, Default)]
pub enum Bucket<T> {
    All,
    List(Vec<T>),

    #[default]
    None
}

impl<T: PartialEq> Bucket<T> {
    pub fn contains(&self, item: &T) -> bool {
        match self {
            Self::All => true,
            Self::List(list) => list.contains(item),
            Self::None => false,
        }
    }
}

impl<T: Clone> Add for Bucket<T> {
    type Output = Self;

    fn add(self, new_modifiers_excluded: Self) -> Self::Output {
        match new_modifiers_excluded.clone() {
            Self::All => Self::All,
            Self::List(mut modifiers_to_add) => {
                match self {
                    Self::All => return Self::All,
                    Self::List(mut modifiers_already_excluded) => {
                        modifiers_already_excluded.append(&mut modifiers_to_add);

                        return Self::List(modifiers_already_excluded)
                    },
                    Self::None => return new_modifiers_excluded.clone(),
                }
            },
            Self::None => return self
        }
    }
}