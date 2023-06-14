use crate::{component_type::ComponentType, mem_utils, archetype_data_layout::ArchetypeDataLayout, archetype::Archetype};

#[derive(Debug)]
pub struct ArchetypeDataPage {
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


    pub fn get_component_ref<'a, T: 'static>(
        &'a self, id: u32, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> &T {
        let entity_index = self.get_entity_index(id);
        let ptr = self.get_component_data_ptr::<T>(entity_index, archetype, layout);
        unsafe { 
            & *ptr
        }
    }

    pub fn get_component_ref_mut<'a, T: 'static>(
        &'a mut self, id: u32, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> &'a mut T {
        let entity_index = self.get_entity_index(id);
        let ptr = self.get_component_data_ptr_mut::<T>(entity_index, archetype, layout);
        unsafe {
            &mut *ptr
        }
    }

    #[inline(always)]
    pub fn get_component_data_entities_row_ptr<T: 'static>(
        &mut self, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> *mut T {
        self.get_component_data_ptr_mut(0, archetype, layout)
    }

    fn get_component_data_ptr<T: 'static>(
        &self, entity_index: usize, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> *const T {

        let component_type = ComponentType::of::<T>();
        let component_index = archetype.find_component_index(component_type.id());

        debug_assert!(entity_index < self.entities_ids.len());
        debug_assert!(
            component_index.is_some(), 
            "There is no component `{}` in the given archetype", std::any::type_name::<T>()
        );

        let component_index = unsafe { component_index.unwrap_unchecked() };

        unsafe {
            let component_buffer_offset = *layout.component_offsets().add(component_index);
            let entity_offset = entity_index * component_type.size();
            self.components_data_ptr.add(component_buffer_offset + entity_offset) as *const T
        }
    }

    #[inline(always)]
    fn get_component_data_ptr_mut<T: 'static>(
        &mut self, entity_index: usize, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> *mut T {
        self.get_component_data_ptr::<T>(entity_index, archetype, layout) as *mut T
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