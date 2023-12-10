use crate::{
    bitvec_utils,
    component_tuple::ComponentTuple,
    query::access::{ComponentQueryAccess, ReadWriteAccess, ReadonlyAccess, WriteAccess},
    Entity, Store, Tag,
};

use super::{
    component_page_iter::ComponentPageIter,
    component_page_iter_view::ComponentPageIterView,
};

pub type ComponentReadWriteQuery<R, W> = ComponentQuery<ReadWriteAccess<R, W>>;
pub type ComponentReadOnlyQuery<R> = ComponentQuery<ReadonlyAccess<R>>;
pub type ComponentWriteQuery<W> = ComponentQuery<WriteAccess<W>>;

pub struct ComponentQuery<T: ComponentQueryAccess> {
    page_views: Vec<ComponentPageIterView<T>>,
}

pub struct ComponentsQueryIter<'a, T: ComponentQueryAccess> {
    store: &'a Store,
    page_views: &'a [ComponentPageIterView<T>],
    current_page_iter: ComponentPageIter<'a, T>,

    current_page_index: usize,
    queried_entities_count: usize,
}

pub struct WithEntitiesIter<'a, T: ComponentQueryAccess> {
    source_iter: ComponentsQueryIter<'a, T>,
    entities_versions: *const u32,
}

pub struct FilterTagIter<'a, T: ComponentQueryAccess> {
    source_iter: WithEntitiesIter<'a, T>,
    entity_to_has_tag: *const u32,
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
                    entity_ids: unsafe { page.entity_id_ptrs() },
                });
                queried_entities_count += page_entities_count;
            }
        }

        let page_iter = if query.page_views.len() > 0 {
            let page_view = query.page_views[0];

            ComponentPageIter::new(page_view)
        } else {
            ComponentPageIter::empty()
        };

        ComponentsQueryIter {
            current_page_iter: page_iter,
            store: &self,
            page_views: &query.page_views,
            current_page_index: 0,
            queried_entities_count,
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for ComponentsQueryIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current_iter = &mut self.current_page_iter;

        if let Some(val) = current_iter.next() {
            return Some(val);
        }

        self.current_page_index += 1;
        if self.current_page_index < self.page_views.len() {
            let page_view =
                unsafe { self.page_views.get_unchecked(self.current_page_index) };

            self.current_page_iter = ComponentPageIter::new(*page_view);

            return self.current_page_iter.next();
        }

        self.current_page_iter = ComponentPageIter::empty();
        return None;
    }
}

impl<'a, T: ComponentQueryAccess> ExactSizeIterator for ComponentsQueryIter<'a, T> {
    fn len(&self) -> usize {
        self.queried_entities_count
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
            let id = self.source_iter.current_page_iter.current_entity_id();
            let version = *self.entities_versions.add(id as usize);

            return (Entity { id, version }, result);
        })
    }
}

impl<'a, Q: ComponentQueryAccess> WithEntitiesIter<'a, Q> {
    pub fn filter_tag<T: Tag + 'static>(self) -> FilterTagIter<'a, Q> {
        let tag_container = &self.source_iter.store.tag_container;
        let entity_to_has_tag = tag_container.has_tag_bitvec::<T>();

        FilterTagIter {
            source_iter: self,
            entity_to_has_tag,
        }
    }
}

impl<'a, Q: ComponentQueryAccess> Iterator for FilterTagIter<'a, Q> {
    type Item = (Entity, Q::AccessOutput<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.source_iter.next().and_then(|(e, o)| unsafe {
            if bitvec_utils::is_bit_on(self.entity_to_has_tag, e.id as usize) {
                Some((e, o))
            } else {
                self.next()
            }
        })
    }
}
