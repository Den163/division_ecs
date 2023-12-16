use std::any::TypeId;

#[derive(Debug, Clone, Copy)]
pub struct ComponentType {
    id: TypeId,
    size: usize,
    align: usize,
    name: &'static str,
}

impl ComponentType {
    pub fn of<T: 'static>() -> Self {
        ComponentType {
            id: TypeId::of::<T>(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
            name: std::any::type_name::<T>(),
        }
    }

    pub(crate) fn new(
        id: TypeId,
        size: usize,
        align: usize,
        name: &'static str,
    ) -> ComponentType {
        ComponentType {
            id,
            size,
            align,
            name,
        }
    }

    #[inline(always)]
    pub fn id(&self) -> TypeId {
        self.id
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.size
    }

    #[inline(always)]
    pub fn align(&self) -> usize {
        self.align
    }

    #[inline(always)]
    pub fn name(&self) -> &'static str {
        self.name
    }
}

impl PartialEq for ComponentType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
