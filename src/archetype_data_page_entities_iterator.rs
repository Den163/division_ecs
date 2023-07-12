use crate::{
    archetype_data_page_view::ArchetypeDataPageView,
    entity_in_archetype_data_page::EntityInArchetypeDataPage,
};

pub(crate) struct ArchetypeDataPageEntitiesIterator<'a> {
    page_views: &'a [ArchetypeDataPageView<'a>],
    current_page_index: usize,
    current_entity_index: usize,
}

impl<'a> ArchetypeDataPageEntitiesIterator<'a> {
    pub fn new(pages: &'a [ArchetypeDataPageView]) -> ArchetypeDataPageEntitiesIterator<'a> {
        ArchetypeDataPageEntitiesIterator {
            page_views: pages,
            current_page_index: 0,
            current_entity_index: 0,
        }
    }
}

impl<'a> Iterator for ArchetypeDataPageEntitiesIterator<'a> {
    type Item = EntityInArchetypeDataPage<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_page_index >= self.page_views.len() {
            return None;
        }

        let page_view = self.page_views[self.current_page_index];
        let entities = page_view.page.entities_ids();
        let entity_index = entities.len();

        if entity_index >= entities.len() {
            return None;
        }

        self.current_page_index += 1;
        self.current_entity_index += 1;

        return Some(EntityInArchetypeDataPage { page_view, entity_index });
    }
}
