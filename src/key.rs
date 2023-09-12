use std::any::TypeId;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum Key {
    Number(usize),
    Type(TypeId),
}

impl Key {
    #[inline(always)]
    pub fn is_numbered(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    #[inline(always)]
    pub fn is_type(&self) -> bool {
        matches!(self, Self::Type(_))
    }
}

impl From<usize> for Key {
    fn from(value: usize) -> Self {
        Self::Number(value)
    }
}
