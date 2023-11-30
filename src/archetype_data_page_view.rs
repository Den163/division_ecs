use crate::{
    archetype_data_layout::ArchetypeDataLayout, archetype_data_page::ArchetypeDataPage,
    tuple::ComponentsTuple, Archetype,
};

#[derive(Clone, Copy)]
pub(crate) struct ArchetypeDataPageView<'a> {
    pub archetype: &'a Archetype,
    pub layout: &'a ArchetypeDataLayout,
    pub page: &'a ArchetypeDataPage,
}

impl<'a> ArchetypeDataPageView<'a> {
    pub fn get_components_refs<T>(&self, page_entity_index: usize) -> T::RefsTuple<'a>
    where
        T: ComponentsTuple,
    {
        let offsets = unsafe {
            T::get_offsets_checked(&self.archetype, self.layout.component_offsets())
        };
        T::get_refs(self.page, page_entity_index, &offsets)
    }

    pub fn get_components_refs_mut<T>(
        &self,
        page_entity_index: usize,
    ) -> T::MutRefsTuple<'a>
    where
        T: ComponentsTuple,
    {
        let offsets = unsafe {
            T::get_offsets_checked(&self.archetype, self.layout.component_offsets())
        };
        T::get_refs_mut(self.page, page_entity_index, &offsets)
    }
}
