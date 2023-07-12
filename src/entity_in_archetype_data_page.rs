use crate::archetype_data_page_view::ArchetypeDataPageView;

pub(crate) struct EntityInArchetypeDataPage<'a> {
    pub page_view: ArchetypeDataPageView<'a>,
    pub entity_index: usize
}