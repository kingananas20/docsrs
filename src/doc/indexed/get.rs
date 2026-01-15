use crate::{Doc, Indexed, Item};

impl Doc<Indexed> {
    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, id: &str) -> Option<&Item> {
        self.0.items.get(id)
    }
}
