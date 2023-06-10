use crate::{component_type::ComponentType, mem_utils, archetype_data_layout::ArchetypeDataLayout, archetype::Archetype};

pub struct ArchetypeDataPage {
    components_data_ptr: *mut usize,
}

impl ArchetypeDataPage {
    pub const PAGE_SIZE_BYTES: usize = 4096;

    pub fn new() -> Self {
        let components_data = mem_utils::alloc::<usize>(Self::PAGE_SIZE_BYTES);
        
        ArchetypeDataPage { components_data_ptr: components_data }
    }

    pub fn get_component_value<T: 'static>(
        &mut self, entity_index: usize, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> &mut T {
        let ptr = self.get_component_data_ptr::<T>(entity_index, archetype, layout);
        unsafe {
            &mut *ptr
        }
    }

    pub fn set_component_value<T: 'static>(
        &mut self, 
        entity_index: usize, 
        archetype: &Archetype,
        layout: &ArchetypeDataLayout,
        value: T
    ) {
        let ptr = self.get_component_data_ptr::<T>(entity_index, archetype, layout);
        unsafe {
            *ptr = value;
        }
    }

    #[inline(always)]
    pub fn get_component_data_row_ptr<T: 'static>(
        &mut self, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> *mut T {
        self.get_component_data_ptr(0, archetype, layout)
    }

    #[inline(always)]
    pub fn get_component_data_ptr<T: 'static>(
        &mut self, entity_index: usize, archetype: &Archetype, layout: &ArchetypeDataLayout
    ) -> *mut T {
        let component_type = ComponentType::of::<T>();
        let component_index = archetype.find_component_index(component_type.id());

        debug_assert!(entity_index < layout.entities_capacity(), "Entity index is out of capacity");
        debug_assert!(
            component_index.is_some(), 
            "There is no component `{}` in the given archetype", std::any::type_name::<T>()
        );

        let component_index = unsafe { component_index.unwrap_unchecked() };

        unsafe {
            let component_buffer_offset = *layout.component_offsets().add(component_index);
            let entity_offset = entity_index * component_type.size();
            self.components_data_ptr.add(component_buffer_offset + entity_offset) as *mut T
        }
    }
}

impl Drop for ArchetypeDataPage {
    fn drop(&mut self) {
        mem_utils::dealloc(self.components_data_ptr, Self::PAGE_SIZE_BYTES);
    }
}