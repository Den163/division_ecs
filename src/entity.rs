#[derive(PartialEq, Debug, Copy, Clone)]
pub struct Entity {
    pub(crate) id: u32,
    pub(crate) version: u32,
}

impl Entity {
    pub const NULL_ID: u32 = u32::MAX;

    #[inline(always)]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[inline(always)]
    pub fn version(&self) -> u32 {
        self.version
    }

    #[inline(always)]
    pub fn null() -> Entity {
        Entity {
            id: Self::NULL_ID,
            version: 0,
        }
    }
}
