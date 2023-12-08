use std::ptr::null;

use crate::{
    archetype_data_page::ArchetypeDataPage,
    component_tuple::ComponentTuple,
    query::access::{ComponentQueryAccess, ReadWriteAccess, ReadonlyAccess, WriteAccess},
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
    next_entity_id: usize,
    entities_count: usize,
}

pub struct WithEntitiesIter<'a, T: ComponentQueryAccess> {
    source_iter: ComponentsQueryIter<'a, T>,
    entities_versions: *const u32,
}

pub(crate) struct PageIterView {
    page: *const ArchetypeDataPage,
    components_offsets_index: usize,
}

pub fn readonly<R: ComponentTuple>() -> ComponentQuery<ReadonlyAccess<R>> {
    ComponentReadOnlyQuery::new()
}

pub fn write<W: ComponentTuple>() -> ComponentQuery<WriteAccess<W>> {
    ComponentWriteQuery::new()
}

pub fn read_write<R: ComponentTuple, W: ComponentTuple>(
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
        let layouts = arch_container.get_layouts();
        let pages = arch_container.get_pages();
        let mut entities_count = 0;

        query.page_views.clear();
        query.components_offsets.clear();

        for arch_idx in 0..archetypes.len() {
            let (arch, layout) = unsafe {
                (
                    archetypes.get_unchecked(arch_idx),
                    layouts.get_unchecked(arch_idx),
                )
            };

            if T::is_archetype_include_types(arch) == false {
                continue;
            }

            query.components_offsets.push(T::get_offsets(arch, layout));

            let components_offsets_index = query.components_offsets.len() - 1;
            let arch_pages = arch_container.get_archetype_page_indices(arch_idx);

            for page_idx in arch_pages {
                let page = &pages[*page_idx];
                let page_entities_count = page.entities_count();
                if page_entities_count == 0 {
                    continue;
                }

                query.page_views.push(PageIterView {
                    page,
                    components_offsets_index,
                });
                entities_count += page_entities_count;
            }
        }

        ComponentsQueryIter {
            current_page_view_index: 0,
            next_entity_id: 0,
            components_offsets: &query.components_offsets,
            page_views: &query.page_views,
            store: self,
            curr_page_ptr: null(),
            entities_count,
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

        if self.next_entity_id >= entities_ids.len() {
            self.current_page_view_index += 1;
            self.next_entity_id = 0;

            if self.current_page_view_index >= page_view_count {
                return None;
            }

            curr_page_view =
                unsafe { page_views.get_unchecked(self.current_page_view_index) };
            curr_page = unsafe { &*curr_page_view.page };
        }

        self.curr_page_ptr = curr_page_view.page;

        unsafe {
            let curr_entity_idx = self.next_entity_id;
            let offsets = self
                .components_offsets
                .get_unchecked(curr_page_view.components_offsets_index);

            self.next_entity_id += 1;

            return Some(T::get_refs(curr_page, curr_entity_idx, offsets));
        }
    }
}

impl<'a, T: ComponentQueryAccess> ExactSizeIterator for ComponentsQueryIter<'a, T> {
    fn len(&self) -> usize {
        self.entities_count
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
            let curr_entity_id = self.source_iter.next_entity_id - 1;
            let page = &*self.source_iter.curr_page_ptr;

            let id = *page.entities_ids().get_unchecked(curr_entity_id);
            let version = *self.entities_versions.add(id as usize);

            return (Entity { id, version }, result);
        })
    }
}
