use super::Storage;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CondId(usize);

impl From<usize> for CondId {
    fn from(n: usize) -> Self {
        Self(n)
    }
}


#[derive(Debug)]
pub struct Conditions(Storage<String>);

impl Conditions {
    pub(crate) fn empty() -> Self {
        Self(Storage::empty())
    }

    pub(crate) fn push(&mut self, cond: Cond) -> StringId {
        CondId(self.0.push(cond))
    }

    pub(crate) fn get(&self, index: CondId) -> Option<&Cond> {
        self.0.get(index.0)
    }
}
