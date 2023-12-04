use crate::{archetype_data_page::ArchetypeDataPage, tuple::ComponentsTuple, Archetype};

#[derive(Clone, Copy)]
pub(crate) struct ArchetypeDataPageView<'a> {
    pub archetype: &'a Archetype,
    pub page: &'a ArchetypeDataPage,
}

impl<'a> ArchetypeDataPageView<'a> {
    pub fn get_components_refs<T: ComponentsTuple>(
        &self,
        page_entity_index: usize,
    ) -> Option<T::RefsTuple<'a>> {
        T::get_offsets(&self.archetype)
            .map(|o| T::get_refs(self.page, page_entity_index, &o))
    }

    pub fn get_components_refs_mut<T: ComponentsTuple>(
        &self,
        page_entity_index: usize,
    ) -> Option<T::MutRefsTuple<'a>> {
        T::get_offsets(&self.archetype)
            .map(|o| T::get_refs_mut(self.page, page_entity_index, &o))
    }

    pub unsafe fn get_components_refs_mut_unchecked<T: ComponentsTuple>(
        &self,
        page_entity_index: usize,
    ) -> T::MutRefsTuple<'a> {
        T::get_refs_mut(
            self.page,
            page_entity_index,
            &T::get_offsets_unchecked(&self.archetype),
        )
    }
}
