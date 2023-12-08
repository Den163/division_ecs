use crate::{archetype_data_page::ArchetypeDataPage, component_tuple::ComponentTuple, Archetype, archetype_layout::ArchetypeLayout};

#[derive(Clone, Copy)]
pub(crate) struct ArchetypeDataPageView<'a> {
    pub archetype: &'a Archetype,
    pub layout: &'a ArchetypeLayout,
    pub page: &'a ArchetypeDataPage,
}

impl<'a> ArchetypeDataPageView<'a> {
    pub fn get_components_refs<T: ComponentTuple>(
        &self,
        page_entity_index: usize,
    ) -> Option<T::RefTuple<'a>> {
        T::get_offsets(self.archetype, self.layout)
            .map(|o| T::get_refs(self.page, page_entity_index, &o))
    }

    pub fn get_components_refs_mut<T: ComponentTuple>(
        &self,
        page_entity_index: usize,
    ) -> Option<T::MutRefTuple<'a>> {
        T::get_offsets(self.archetype, self.layout)
            .map(|o| T::get_refs_mut(self.page, page_entity_index, &o))
    }

    pub unsafe fn get_components_refs_mut_unchecked<T: ComponentTuple>(
        &self,
        page_entity_index: usize,
    ) -> T::MutRefTuple<'a> {
        T::get_refs_mut(
            self.page,
            page_entity_index,
            &T::get_offsets_unchecked(self.archetype, self.layout),
        )
    }
}
