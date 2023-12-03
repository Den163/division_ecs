use std::ptr::null;

use crate::{
    archetype_data_page::ArchetypeDataPage,
    component_query_access::{
        ComponentQueryAccess, ReadWriteAccess, ReadonlyAccess, WriteAccess,
    },
    tuple::ComponentsTuple,
    Entity, Store,
};

pub type ComponentReadWriteQuery<R, W> = ComponentQuery<ReadWriteAccess<R, W>>;
pub type ComponentReadOnlyQuery<R> = ComponentQuery<ReadonlyAccess<R>>;
pub type ComponentWriteQuery<W> = ComponentQuery<WriteAccess<W>>;

pub struct ComponentQuery<T: ComponentQueryAccess> {
    page_views: Vec<PageIterView>,
    components_offsets: Vec<T::OffsetsTuple>,
}

pub struct ComponentsQueryIter<'a, T: ComponentQueryAccess> {
    store: &'a Store,
    page_views: &'a [PageIterView],
    components_offsets: &'a [T::OffsetsTuple],
    curr_page_ptr: *const ArchetypeDataPage,

    current_page_view_index: usize,
    next_entity_index: usize,
}

pub struct WithEntitiesIter<'a, T: ComponentQueryAccess> {
    source_iter: ComponentsQueryIter<'a, T>,
    entities_versions: *const u32,
}

pub(crate) struct PageIterView {
    page: *const ArchetypeDataPage,
    components_offsets_index: usize,
}

pub fn readonly<R: ComponentsTuple>() -> ComponentQuery<ReadonlyAccess<R>> {
    ComponentReadOnlyQuery::new()
}

pub fn write<W: ComponentsTuple>() -> ComponentQuery<WriteAccess<W>> {
    ComponentWriteQuery::new()
}

pub fn read_write<R: ComponentsTuple, W: ComponentsTuple>(
) -> ComponentQuery<ReadWriteAccess<R, W>> {
    ComponentReadWriteQuery::new()
}

impl<T: ComponentQueryAccess> ComponentQuery<T> {
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
            next_entity_index: 0,
            components_offsets: &query.components_offsets,
            page_views: &query.page_views,
            store: self,
            curr_page_ptr: null()
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for ComponentsQueryIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let page_views = self.page_views;
        let page_view_count = self.page_views.len();

        if self.current_page_view_index >= page_view_count {
            return None;
        }

        let mut curr_page_view =
            unsafe { page_views.get_unchecked(self.current_page_view_index) };
        let mut curr_page = unsafe { &*curr_page_view.page };
        let entities_ids = curr_page.entities_ids();

        if self.next_entity_index >= entities_ids.len() {
            self.current_page_view_index += 1;
            self.next_entity_index = 0;

            if self.current_page_view_index >= page_view_count {
                return None;
            }

            curr_page_view =
                unsafe { page_views.get_unchecked(self.current_page_view_index) };
            curr_page = unsafe { &*curr_page_view.page };
            // entities_ids = curr_page.entities_ids();
        }

        self.curr_page_ptr = curr_page_view.page;

        unsafe {
            let curr_entity_idx = self.next_entity_index;
            // let id = *entities_ids.get_unchecked(curr_entity_idx);
            // let version = *self.entities_versions.get_unchecked(id as usize);
            let offsets = self
                .components_offsets
                .get_unchecked(curr_page_view.components_offsets_index);

            self.next_entity_index += 1;

            return Some(T::get_refs(curr_page, curr_entity_idx, offsets));
        }
    }
}

impl<'a, T: ComponentQueryAccess> ComponentsQueryIter<'a, T> {
    pub fn with_entities(self) -> WithEntitiesIter<'a, T> {
        let entities_versions = self.store.entities_container.entity_versions();

        WithEntitiesIter {
            source_iter: self,
            entities_versions,
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for WithEntitiesIter<'a, T> {
    type Item = (Entity, T::AccessOutput<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.source_iter.next().map(|result| unsafe {
            let entity_index = self.source_iter.next_entity_index - 1;
            let page = &*self.source_iter.curr_page_ptr;

            let id = *page.entities_ids().get_unchecked(entity_index);
            let version = *self.entities_versions.add(id as usize);

            return (Entity { id, version }, result);
        })
    }
}
