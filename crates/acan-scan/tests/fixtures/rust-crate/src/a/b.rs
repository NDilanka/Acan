use crate::c::helper;

pub struct Thing;

impl Thing {
    pub fn label(&self) -> &'static str {
        helper();
        "thing"
    }

    fn hidden(&self) {}
}

pub fn make() -> Thing {
    Thing
}
