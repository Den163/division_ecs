use crate::{mem_utils, archetype_data_layout::ArchetypeDataLayout, archetype::Archetype};

#[derive(Debug)]
pub(crate) struct ArchetypeDataPage {
    entities_ids: Vec<u32>,
    entities_layout_capacity: usize,

    components_data_ptr: *mut u8,
}

impl ArchetypeDataPage {
    pub const PAGE_SIZE_BYTES: usize = 4096;

    pub fn new() -> Self {
        let components_data_ptr = mem_utils::alloc(Self::PAGE_SIZE_BYTES);
        
        ArchetypeDataPage { components_data_ptr, entities_layout_capacity: 0, entities_ids: Vec::new() }
    }

    pub fn set_layout(&mut self, layout: &ArchetypeDataLayout) {
        let capacity = layout.entities_capacity();
        self.entities_ids.reserve(capacity);
        self.entities_layout_capacity = capacity;
    }

    #[inline(always)]
    pub fn entities_count(&self) -> usize { 
        self.entities_ids.len()
    }

    #[inline(always)]
    pub fn entities_capacity(&self) -> usize {
        self.entities_layout_capacity
    }

    #[inline(always)]
    pub fn has_free_space(&self) -> bool {
        self.entities_count() < self.entities_capacity()
    }

    pub fn push_entity_id(&mut self, id: u32) {
        debug_assert!(self.has_free_space());
        match self.entities_ids.binary_search(&id) {
            Ok(_) => panic!("There is already entity with id {} in the page", id),
            Err(new_index) => self.entities_ids.insert(new_index, id),
        }
    }

    pub fn remove_entity_id(&mut self, id: u32) {
        let index = self.get_entity_index(id);
        self.entities_ids.remove(index);
    }

    pub fn get_component_by_entity_id_ref<'a, T: 'static>(
        &'a self, id: u32, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> &T {
        let component_index = Self::get_valid_component_index::<T>(archetype);
        self.get_indexed_component_ref(
            component_index, self.get_entity_index(id), layout)
    }

    pub fn get_component_by_entity_id_ref_mut<'a, T: 'static>(
        &'a mut self, id: u32, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> &'a mut T {
        let component_index = Self::get_valid_component_index::<T>(archetype);
        self.get_indexed_component_ref_mut(
            component_index, self.get_entity_index(id), layout)
    }

    pub fn get_indexed_component_ref<'a, T: 'static>(
        &'a self, component_index: usize, entity_index: usize, layout: &ArchetypeDataLayout
    ) -> &'a T {
        let ptr = self.get_component_data_ptr(entity_index, component_index, layout);
        unsafe {
            & *ptr
        }
    }

    pub fn get_indexed_component_ref_mut<'a, T: 'static>(
        &'a mut self, component_index: usize, entity_index: usize, layout: &ArchetypeDataLayout
    ) -> &'a mut T {
        let ptr = self.get_component_data_ptr(entity_index, component_index, layout);
        unsafe {
            &mut *ptr
        }
    }

    fn get_valid_component_index<T: 'static>(archetype: &Archetype) -> usize {
        let component_index = archetype.find_component_index(std::any::TypeId::of::<T>());
        debug_assert!(
            component_index.is_some(), 
            "There is no component {} in the given archetype", std::any::type_name::<T>()
        );

        unsafe { component_index.unwrap_unchecked() }
    }

    #[inline(always)]
    fn get_component_data_ptr<T: 'static>(
        &self, entity_index: usize, component_index: usize, layout: &ArchetypeDataLayout
    ) -> *mut T {
        unsafe {
            let ref this = self;
            let component_buffer_offset = *layout.component_offsets().add(component_index);
            let component_data_row_ptr = this.components_data_ptr.add(component_buffer_offset);
            let entity_offset = entity_index * std::mem::size_of::<T>();

            component_data_row_ptr.add(entity_offset) as *mut T
        }
    }

    #[inline(always)]
    fn get_entity_index(&self, id: u32) -> usize {
        match self.entities_ids.binary_search(&id) {
            Ok(index) => index,
            Err(_) => panic!("There is no entity with id {}", id),
        }
    }
}

impl Drop for ArchetypeDataPage {
    fn drop(&mut self) {
        mem_utils::dealloc(self.components_data_ptr, Self::PAGE_SIZE_BYTES);
    }
}