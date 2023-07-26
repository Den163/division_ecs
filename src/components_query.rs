use crate::{archetype_data_page::ArchetypeDataPage, Entity, Store, components_query_access::ComponentsQueryAccess};

pub trait QueryIntoIter<T>
where
    T: ComponentsQueryAccess
{
    fn into_iter<'a, 'b: 'a>(&'a self, query: &'b mut ComponentsQuery<T>) -> ComponentsQueryIter<'a, T>;
}

pub struct ComponentsQuery<T>
where
    T: ComponentsQueryAccess
{
    page_views: Vec<PageIterView>,
    components_offsets: Vec<T::OffsetsTuple>,
}

pub type ComponentsReadWriteQuery<TRead, TWrite> = ComponentsQuery<(TRead, TWrite)>;
pub type ComponentsReadOnlyQuery<TRead> = ComponentsQuery<(TRead, ())>;
pub type ComponentsWriteQuery<TWrite> = ComponentsQuery<((), TWrite)>;

pub struct ComponentsQueryIter<'a, T>
where
    T: ComponentsQueryAccess,
{
    page_views: &'a [PageIterView],
    components_offsets: &'a [T::OffsetsTuple],
    entities_versions: &'a [u32],

    current_page_view_index: usize,
    current_entity_index: usize,
}

struct PageIterView {
    page: *const ArchetypeDataPage,
    components_offsets_index: usize,
}

impl<T> ComponentsQuery<T>
where
    T: ComponentsQueryAccess
{
    pub fn new() -> Self {
        ComponentsQuery { page_views: Vec::new(), components_offsets: Vec::new() }
    }
}

impl<T> QueryIntoIter<T> for Store
where
    T: ComponentsQueryAccess,
{
    fn into_iter<'a, 'b: 'a>(&'a self, query: &'b mut ComponentsQuery<T>) -> ComponentsQueryIter<'a, T> {
        let arch_container = &self.archetypes_container;
        let archetypes = arch_container.get_archetypes();
        let layouts = arch_container.get_layouts();
        let pages = arch_container.get_pages();

        query.page_views.clear();
        query.components_offsets.clear();

        for (arch_idx, arch) in archetypes.into_iter().enumerate() {
            if T::is_archetype_include_types(arch) == false {
                continue;
            }

            let offsets = layouts[arch_idx].component_offsets();
            query
                .components_offsets
                .push(T::get_offsets(arch, offsets));

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
            entities_versions: self.entities_container.get_entity_versions(),
        }
    }
}

impl<'a, T> Iterator for ComponentsQueryIter<'a, T>
where
    T: ComponentsQueryAccess,
{
    type Item = (Entity, T::AccessOutput<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        let page_views = self.page_views;
        let page_view_count = self.page_views.len();

        if self.current_page_view_index >= page_view_count {
            return None;
        }

        let mut curr_page_view = &page_views[self.current_page_view_index];
        let mut curr_page = unsafe { &*curr_page_view.page };
        let mut entities_ids = curr_page.entities_ids();

        if self.current_entity_index >= entities_ids.len() {
            self.current_page_view_index += 1;
            self.current_entity_index = 0;

            if self.current_page_view_index >= page_view_count {
                return None;
            }

            curr_page_view = unsafe { page_views.get_unchecked(self.current_page_view_index) };
            curr_page = unsafe { &*curr_page_view.page };
            entities_ids = curr_page.entities_ids();
        }

        unsafe {
            let curr_entity_idx = self.current_entity_index;
            let id = *entities_ids.get_unchecked(curr_entity_idx);
            let version = *self.entities_versions.get_unchecked(id as usize);
            let offsets = self.components_offsets.get_unchecked(curr_page_view.components_offsets_index);

            self.current_entity_index += 1;

            return Some((
                Entity { id, version },
                T::get_refs(curr_page, curr_entity_idx, offsets),
            ));
        }
    }
}
