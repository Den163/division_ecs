use crate::{Entity, component_type::ComponentType, mem_utils, archetype::Archetype};


pub(crate) struct ArchetypeDataPage {
    components_data: *mut usize,
}

impl ArchetypeDataPage {
    pub const PAGE_SIZE_BYTES: usize = 4096;

    pub fn new(components: &[ComponentType]) -> Self {
        let components_data = mem_utils::alloc::<usize>(Self::PAGE_SIZE_BYTES);
        
        ArchetypeDataPage {
            components_data,
        }
    }

    pub fn set_component_value<T: 'static, TU>(&mut self, entity_index: usize, component_index: usize, value: T) {
        let component_type = ComponentType::of::<T>();

        unsafe {
            let component_buffer_offset = *self.component_offsets.add(component_index);
            let entity_offset = entity_index * component_type.size();
            let data_ptr = self.components_data.add(component_buffer_offset + entity_offset) as *mut T;
            *data_ptr = value;
        }
    }
}

impl Drop for ArchetypeDataPage {
    fn drop(&mut self) {
        mem_utils::dealloc(self.components_data, Self::PAGE_SIZE_BYTES);
    }
}