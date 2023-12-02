use crate::{
    archetype_data_page::ArchetypeDataPage,
    component_query_access::ComponentQueryAccess, Entity, Store,
};

pub type ComponentReadWriteQuery<TRead, TWrite> = ComponentQuery<(TRead, TWrite)>;
pub type ComponentReadOnlyQuery<TRead> = ComponentQuery<(TRead, ())>;
pub type ComponentWriteQuery<TWrite> = ComponentQuery<((), TWrite)>;

pub struct ComponentQuery<T: ComponentQueryAccess> {
    page_views: Vec<PageIterView>,
    components_offsets: Vec<T::OffsetsTuple>,
}

pub struct ComponentsQueryIter<'a, T: ComponentQueryAccess> {
    page_views: &'a [PageIterView],
    components_offsets: &'a [T::OffsetsTuple],
    entities_versions: &'a [u32],

    current_page_view_index: usize,
    current_entity_index: usize,
}

pub(crate) struct PageIterView {
    page: *const ArchetypeDataPage,
    components_offsets_index: usize,
}

impl<T: ComponentQueryAccess> ComponentQuery<T>
{
    pub fn new() -> Self {
        ComponentQuery {
            page_views: Vec::new(),
            components_offsets: Vec::new(),
        }
    }
}

impl Store {
    pub fn component_query_iter<'a, 'b: 'a, T: ComponentQueryAccess>(
        &'a self,
        query: &'b mut ComponentQuery<T>,
    ) -> ComponentsQueryIter<'a, T> {
        let arch_container = &self.archetypes_container;
        let archetypes = arch_container.get_archetypes();
        let pages = arch_container.get_pages();

        query.page_views.clear();
        query.components_offsets.clear();

        for (arch_idx, arch) in archetypes.into_iter().enumerate() {
            if T::is_archetype_include_types(arch) == false {
                continue;
            }

            query.components_offsets.push(T::get_offsets(arch));
            
            let components_offsets_index = query.components_offsets.len() - 1;
            let arch_pages = arch_container.get_archetype_page_indices(arch_idx);

            for page_idx in arch_pages {
                let page = &pages[*page_idx];
                if page.entities_count() == 0 {
                    continue;
                }

                query.page_views.push(PageIterView {
                    page,
                    components_offsets_index,
                });
            }
        }

        ComponentsQueryIter {
            current_page_view_index: 0,
            current_entity_index: 0,
            components_offsets: &query.components_offsets,
            page_views: &query.page_views,
            entities_versions: self.entities_container.entity_versions(),
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for ComponentsQueryIter<'a, T>
{
    type Item = (Entity, T::AccessOutput<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let page_views = self.page_views;
        let page_view_count = self.page_views.len();

        if self.current_page_view_index >= page_view_count {
            return None;
        }

        let mut curr_page_view = unsafe {
            page_views.get_unchecked(self.current_page_view_index)
        };
        let mut curr_page = unsafe { &*curr_page_view.page };
        let mut entities_ids = curr_page.entities_ids();

        if self.current_entity_index >= entities_ids.len() {
            self.current_page_view_index += 1;
            self.current_entity_index = 0;

            if self.current_page_view_index >= page_view_count {
                return None;
            }

            curr_page_view =
                unsafe { page_views.get_unchecked(self.current_page_view_index) };
            curr_page = unsafe { &*curr_page_view.page };
            entities_ids = curr_page.entities_ids();
        }

        unsafe {
            let curr_entity_idx = self.current_entity_index;
            let id = *entities_ids.get_unchecked(curr_entity_idx);
            let version = *self.entities_versions.get_unchecked(id as usize);
            let offsets = self
                .components_offsets
                .get_unchecked(curr_page_view.components_offsets_index);

            self.current_entity_index += 1;

            return Some((
                Entity { id, version },
                T::get_refs(curr_page, curr_entity_idx, offsets),
            ));
        }
    }
}
