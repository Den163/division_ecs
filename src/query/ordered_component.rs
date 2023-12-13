use std::{marker::PhantomData, ops::Range};

use crate::{
    component_tuple::ComponentTuple, Entity, ReadWriteAccess, ReadonlyAccess, Store, Tag,
    WriteAccess,
};

use super::{
    access::ComponentQueryAccess, component_page_iter_view::ComponentPageIterView,
};

pub struct OrderedComponentQuery<O: Tag, T: ComponentQueryAccess> {
    index_in_page_ranges: Vec<Range<u32>>,
    range_to_page_views: Vec<ComponentPageIterView<T>>,
    phantom_order_group: PhantomData<O>,
}

pub struct OrderedComponentQueryIter<'a, T: ComponentQueryAccess> {
    store: &'a Store,

    entity_ranges: &'a [Range<u32>],
    range_pages: &'a [ComponentPageIterView<T>],

    next_range_index: u32,
    current_index_in_page: u32,
    next_offset_from_range: u32,
}

pub struct WithEntitiesIter<'a, T: ComponentQueryAccess> {
    source_iter: OrderedComponentQueryIter<'a, T>,
    entity_versions: *const u32,
}

pub type ReadonlyOrderedComponentQuery<O, C> =
    OrderedComponentQuery<O, ReadonlyAccess<C>>;
pub type ReadWriteOrderedComponentQuery<O, CR, CW> =
    OrderedComponentQuery<O, ReadWriteAccess<CR, CW>>;
pub type WriteOrderedComponentQuery<O, C> = OrderedComponentQuery<O, WriteAccess<C>>;

pub fn readonly<O: Tag, C: ComponentTuple>() -> ReadonlyOrderedComponentQuery<O, C> {
    OrderedComponentQuery::new()
}

pub fn read_write<O: Tag, CR: ComponentTuple, CW: ComponentTuple>(
) -> ReadWriteOrderedComponentQuery<O, CR, CW> {
    OrderedComponentQuery::new()
}

pub fn write<O: Tag, C: ComponentTuple>() -> WriteOrderedComponentQuery<O, C> {
    OrderedComponentQuery::new()
}

impl<O: Tag, T: ComponentQueryAccess> OrderedComponentQuery<O, T> {
    pub fn new() -> Self {
        Self {
            index_in_page_ranges: Vec::new(),
            range_to_page_views: Vec::new(),
            phantom_order_group: PhantomData::default(),
        }
    }
}

impl Store {
    pub fn ordered_query_iter<'a, 'b: 'a, O: Tag, T: ComponentQueryAccess>(
        &'a self,
        query: &'b mut OrderedComponentQuery<O, T>,
    ) -> OrderedComponentQueryIter<'a, T> {
        query.index_in_page_ranges.clear();
        query.range_to_page_views.clear();

        let group_index = match self.order_group_container.get_group_index::<O>() {
            Some(i) => i,
            None => return self.iter_from_query(query),
        };

        let id_to_next_map = unsafe {
            self.order_group_container
                .get_id_to_next_in_group_map_unchecked(group_index)
        };

        let mut curr_entity = unsafe {
            self.order_group_container
                .get_first_id_in_group_unchecked(group_index)
        };

        while curr_entity != Entity::NULL_ID {
            let (curr_page_index, curr_index_in_page) = unsafe {
                (
                    self.get_page_index_unchecked(curr_entity),
                    self.get_index_in_page_unchecked(curr_entity),
                )
            };

            let (page, component_offsets) = unsafe {
                let page = self
                    .archetypes_container
                    .get_page_by_index_unchecked(curr_page_index);
                let curr_arch_index = self
                    .archetypes_container
                    .get_archetype_index_by_page(curr_page_index as usize);
                let (arch, layout) = self
                    .archetypes_container
                    .get_archetype_with_layout_unchecked(curr_arch_index);

                if !T::is_archetype_include_types(arch) {
                    curr_entity = *id_to_next_map.add(curr_entity as usize);
                    continue;
                }

                let offsets = T::get_offsets(arch, layout);

                (page, offsets)
            };

            let range_start = curr_index_in_page;
            let mut range_end = curr_index_in_page + 1;

            let mut next_entity = unsafe { *id_to_next_map.add(curr_entity as usize) };
            while next_entity != Entity::NULL_ID {
                let (next_page_index, next_index_in_page) = unsafe {
                    (
                        self.get_page_index_unchecked(next_entity),
                        self.get_index_in_page_unchecked(next_entity),
                    )
                };

                if (next_page_index != curr_page_index)
                    | (next_index_in_page != range_end)
                {
                    break;
                }

                next_entity = unsafe { *id_to_next_map.add(next_entity as usize) };
                range_end += 1;
            }

            query.index_in_page_ranges.push(range_start..range_end);
            query
                .range_to_page_views
                .push(unsafe { ComponentPageIterView::new(page, &component_offsets) });

            curr_entity = next_entity
        }

        self.iter_from_query(query)
    }

    fn iter_from_query<'a, 'b: 'a, O: Tag, T: ComponentQueryAccess>(
        &'a self,
        query: &'b mut OrderedComponentQuery<O, T>,
    ) -> OrderedComponentQueryIter<'a, T> {
        OrderedComponentQueryIter {
            store: self,

            entity_ranges: &query.index_in_page_ranges,
            range_pages: &query.range_to_page_views,
            next_range_index: 0,
            current_index_in_page: 0,
            next_offset_from_range: 0,
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for OrderedComponentQueryIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_range_index as usize >= self.entity_ranges.len() {
            return None;
        }

        let curr_range_index = self.next_range_index as usize;
        let current_range = unsafe { self.entity_ranges.get_unchecked(curr_range_index) };
        self.current_index_in_page = current_range.start + self.next_offset_from_range;

        if self.current_index_in_page >= current_range.end - 1 {
            self.next_range_index += 1;
            self.next_offset_from_range = 0;
        } else {
            self.next_offset_from_range += 1;
        }

        let page_view = unsafe { self.range_pages.get_unchecked(curr_range_index) };
        let ptrs = T::add_to_ptrs(&page_view.ptrs, self.current_index_in_page as usize);

        return Some(T::ptrs_to_refs(ptrs));
    }
}

impl<'a, T: ComponentQueryAccess> OrderedComponentQueryIter<'a, T> {
    pub fn with_entities(self) -> WithEntitiesIter<'a, T> {
        WithEntitiesIter {
            entity_versions: self.store.entities_container.entity_versions(),
            source_iter: self
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for WithEntitiesIter<'a, T> {
    type Item = (Entity, T::AccessOutput<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.source_iter.next().map(|components| unsafe {
            let page = *self
                .source_iter
                .range_pages
                .get_unchecked(self.source_iter.next_range_index.saturating_sub(1) as usize);

            let id = *page
                .entity_ids
                .add(self.source_iter.current_index_in_page as usize);
            (
                Entity {
                    id,
                    version: *self.entity_versions.add(id as usize),
                },
                components,
            )
        })
    }
}
