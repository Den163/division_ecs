use std::any::TypeId;

#[derive(Debug, Clone, Copy)]
pub struct ComponentType {
    id: TypeId,
    size: usize,
    align: usize,
}

impl ComponentType {
    pub fn of<T: 'static>()  -> Self {
        ComponentType {
            id: TypeId::of::<T>(),
            size: std::mem::size_of::<T>(),
            align: std::mem::align_of::<T>(),
        }
    }

    pub(crate) fn new(id: TypeId, size: usize, align: usize) -> ComponentType {
        ComponentType { id, size, align }
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
}
