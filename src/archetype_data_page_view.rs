use crate::{
    archetype_data_layout::ArchetypeDataLayout, 
    archetype_data_page::ArchetypeDataPage, 
    Archetype,
};

#[derive(Clone, Copy)]
pub(crate) struct ArchetypeDataPageView<'a> {
    pub archetype: &'a Archetype,
    pub layout: &'a ArchetypeDataLayout,
    pub page: &'a ArchetypeDataPage,
}

impl<'a> ArchetypeDataPageView<'a> {
    pub fn get_component_offset<T: 'static>(&self) -> usize {
        let component_index = self.archetype.find_component_index(
            std::any::TypeId::of::<T>()
        );
        assert!(
            component_index.is_some(),
            "There is no component {} in the given archetype",
            std::any::type_name::<T>()
        );

        unsafe {
            let component_index = component_index.unwrap_unchecked();
            *self.layout.component_offsets().add(component_index)
        }
    }

    pub fn get_component_slice<T: 'static>(&'a self) -> &'a [T] {
        unsafe {
            &*std::ptr::slice_from_raw_parts(
                self.get_component_ptr(0), 
                self.page.entities_count()
            )
        }
    }

    pub fn get_component_slice_mut<T: 'static>(&'a self) -> &'a mut [T] {
        unsafe {
            &mut *std::ptr::slice_from_raw_parts_mut(
                self.get_component_ptr_mut(0), 
                self.page.entities_count()
            )
        }
    }
    
    #[inline(always)]
    pub fn get_component_ptr<T: 'static>(&self, page_entity_index: usize) -> *const T {
        self.page.get_component_data_ptr(
            page_entity_index, 
            self.get_component_offset::<T>(), 
            std::mem::size_of::<T>()
        ) as *const T
    }

    #[inline(always)]    
    pub fn get_component_ptr_mut<T: 'static>(&self, page_entity_index: usize) -> *mut T {
        self.page.get_component_data_ptr_mut(
            page_entity_index, 
            self.get_component_offset::<T>(), 
            std::mem::size_of::<T>()
        ) as *mut T
    }
}
