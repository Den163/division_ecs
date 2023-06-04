#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) version: u32
}

impl Entity {
    #[inline(always)]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline(always)]
    pub fn version(&self) -> u32 {
        self.version
    }
}