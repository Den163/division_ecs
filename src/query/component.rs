use crate::{
    component_tuple::ComponentTuple,
    query::access::{ComponentQueryAccess, ReadWriteAccess, ReadonlyAccess, WriteAccess},
    Entity, Store,
};

use super::component_page_iter_view::ComponentPageIterView;

pub type ComponentReadWriteQuery<R, W> = ComponentQuery<ReadWriteAccess<R, W>>;
pub type ComponentReadOnlyQuery<R> = ComponentQuery<ReadonlyAccess<R>>;
pub type ComponentWriteQuery<W> = ComponentQuery<WriteAccess<W>>;

pub struct ComponentQuery<T: ComponentQueryAccess> {
    page_views: Vec<ComponentPageIterView<T>>,
}

pub struct ComponentsQueryIter<'a, T: ComponentQueryAccess> {
    versions: *const u32,

    page_views: &'a [ComponentPageIterView<T>],

    current_page_index: usize,
    current_entity_index: usize,
    current_entity_id: u32,

    queried_entities_count: usize,
}

pub struct WithEntitiesIter<'a, T: ComponentQueryAccess> {
    source_iter: ComponentsQueryIter<'a, T>,
    entities_versions: *const u32,
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
        }
    }
}

impl Store {
    pub fn component_query_iter<'a, 'b: 'a, T: ComponentQueryAccess>(
        &'a self,
        query: &'b mut ComponentQuery<T>,
    ) -> ComponentsQueryIter<'a, T> {
        let arch_container = &self.archetypes_container;
        let pages = arch_container.get_pages();
        let mut queried_entities_count = 0;

        query.page_views.clear();

        for arch_idx in 0..arch_container.get_archetypes().len() {
            let (arch, layout) = unsafe {
                self.archetypes_container
                    .get_archetype_with_layout_unchecked(arch_idx)
            };

            if T::is_archetype_include_types(arch) == false {
                continue;
            }

            let component_offsets = T::get_offsets(arch, layout);

            let arch_pages = arch_container.get_archetype_page_indices(arch_idx);

            for page_idx in arch_pages {
                let page = &pages[*page_idx];
                let page_entities_count = page.entity_count();
                if page_entities_count == 0 {
                    continue;
                }

                query.page_views.push(ComponentPageIterView {
                    ptrs: T::get_ptrs(page, &component_offsets),
                    entity_count: page.entity_count(),
                    entity_ids: unsafe { page.entity_id_ptrs() }
                });
                queried_entities_count += page_entities_count;
            }
        }

        ComponentsQueryIter {
            versions: self.entities_container.entity_versions(),

            page_views: &query.page_views,

            current_page_index: 0,
            current_entity_index: 0,
            current_entity_id: 0,

            queried_entities_count,
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for ComponentsQueryIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_page_index >= self.page_views.len() {
            return None;
        }

        let page_view = unsafe {
            self.page_views.get_unchecked(self.current_page_index)
        };

        self.current_entity_id = unsafe {
            *page_view.entity_ids.add(self.current_entity_index)
        };

        let ptrs = T::add_to_ptrs(&page_view.ptrs, self.current_entity_index);
        let result = T::ptrs_to_refs(ptrs);

        self.current_entity_index += 1;
        if self.current_entity_index >= page_view.entity_count {
            self.current_page_index += 1;
            self.current_entity_index = 0;
        }

        return Some(result);
    }
}

impl<'a, T: ComponentQueryAccess> ExactSizeIterator for ComponentsQueryIter<'a, T> {
    fn len(&self) -> usize {
        self.queried_entities_count
    }
}

impl<'a, T: ComponentQueryAccess> ComponentsQueryIter<'a, T> {
    pub fn with_entities(self) -> WithEntitiesIter<'a, T> {
        let entities_versions = self.versions;

        WithEntitiesIter {
            source_iter: self,
            entities_versions: entities_versions,
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for WithEntitiesIter<'a, T> {
    type Item = (Entity, T::AccessOutput<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.source_iter.next().map(|result| unsafe {
            let id = self.source_iter.current_entity_id;
            let version = *self.entities_versions.add(id as usize);

            return (Entity { id, version }, result);
        })
    }
}
