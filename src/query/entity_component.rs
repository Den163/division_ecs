use std::ops::Range;

use crate::{
    archetype_data_page::ArchetypeDataPage,
    query::access::{ComponentQueryAccess, ReadWriteAccess, ReadonlyAccess, WriteAccess},
    tuple::ComponentsTuple,
    Entity, Store,
};

pub type EntityComponentReadWriteQuery<R, W> =
    EntityComponentQuery<ReadWriteAccess<R, W>>;
pub type EntityComponentReadOnlyQuery<R> = EntityComponentQuery<ReadonlyAccess<R>>;
pub type EntityComponentWriteQuery<W> = EntityComponentQuery<WriteAccess<W>>;

pub struct EntityComponentQuery<T: ComponentQueryAccess> {
    entity_index_ranges: Vec<Range<usize>>,
    range_to_component_offsets: Vec<T::OffsetsTuple>,
    range_to_pages: Vec<*const ArchetypeDataPage>,
}

pub struct EntityComponentQueryIter<'a, T: ComponentQueryAccess> {
    store: &'a Store,
    entities: &'a [Entity],
    entity_ranges: &'a [Range<usize>],
    range_component_offsets: &'a [T::OffsetsTuple],
    range_pages: &'a [*const ArchetypeDataPage],
    current_entity_index: usize,

    next_range_index: usize,
    next_offset_from_range: usize,
}

pub struct WithEntitiesIter<'a, T: ComponentQueryAccess> {
    source_iter: EntityComponentQueryIter<'a, T>,
}

impl<T: ComponentQueryAccess> EntityComponentQuery<T> {
    pub fn new() -> Self {
        Self {
            entity_index_ranges: Vec::new(),
            range_to_component_offsets: Vec::new(),
            range_to_pages: Vec::new(),
        }
    }
}

pub fn readonly<R: ComponentsTuple>() -> EntityComponentReadOnlyQuery<R> {
    EntityComponentQuery::new()
}

pub fn write<W: ComponentsTuple>() -> EntityComponentWriteQuery<W> {
    EntityComponentQuery::new()
}

pub fn read_write<R: ComponentsTuple, W: ComponentsTuple>(
) -> EntityComponentReadWriteQuery<R, W> {
    EntityComponentQuery::new()
}

impl Store {
    pub fn entity_component_query_iter<'a, 'b: 'a, T: ComponentQueryAccess>(
        &'a self,
        entities: &'b [Entity],
        query: &'b mut EntityComponentQuery<T>,
    ) -> EntityComponentQueryIter<'a, T> {
        query.entity_index_ranges.clear();
        query.range_to_pages.clear();
        query.range_to_component_offsets.clear();

        let mut range_start = 0;
        while range_start < entities.len() {
            let range_entity = entities[range_start];

            if self.is_valid_entity_for_query::<T>(range_entity) == false {
                range_start += 1;
                continue;
            }

            let range_page_index =
                unsafe { self.get_page_index_unchecked(range_entity.id) as usize };
            let arch_index = self
                .archetypes_container
                .get_archetype_index_by_page(range_page_index);
            let arch = &self.archetypes_container.get_archetypes()[arch_index];
            let page = unsafe {
                self.archetypes_container
                    .get_pages()
                    .get_unchecked(range_page_index)
            };
            let comp_offsets = T::get_offsets(&arch);

            let mut range_end = range_start + 1;
            while range_end < entities.len() {
                let inner_entity = entities[range_end];
                if self.is_valid_entity_for_query::<T>(inner_entity) == false {
                    break;
                }

                let page_index =
                    unsafe { self.get_page_index_unchecked(inner_entity.id) as usize };
                if range_page_index != page_index {
                    break;
                }

                range_end += 1;
            }

            query.entity_index_ranges.push(range_start..range_end);
            query.range_to_component_offsets.push(comp_offsets);
            query.range_to_pages.push(page as *const ArchetypeDataPage);

            range_start = range_end;
        }

        EntityComponentQueryIter {
            store: &self,
            entities: &entities,
            entity_ranges: &query.entity_index_ranges,
            range_pages: &query.range_to_pages,
            range_component_offsets: &query.range_to_component_offsets,
            next_range_index: 0,
            next_offset_from_range: 0,
            current_entity_index: 0,
        }
    }

    #[inline(always)]
    fn is_valid_entity_for_query<T: ComponentQueryAccess>(&self, entity: Entity) -> bool {
        self.get_entity_archetype(entity)
            .map(|arch| T::is_archetype_include_types(arch))
            .unwrap_or(false)
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for EntityComponentQueryIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_range_index >= self.entity_ranges.len() {
            return None;
        }

        let current_range =
            unsafe { self.entity_ranges.get_unchecked(self.next_range_index) };
        self.current_entity_index = current_range.start + self.next_offset_from_range;

        let (index_in_page, page, offsets) = unsafe {
            let current_entity = self.entities.get_unchecked(self.current_entity_index);
            let page = *self.range_pages.get_unchecked(self.next_range_index);
            (
                self.store.get_index_in_page_unchecked(current_entity.id),
                &*page,
                self.range_component_offsets
                    .get_unchecked(self.next_range_index),
            )
        };

        if self.current_entity_index >= current_range.end - 1 {
            self.next_range_index += 1;
            self.next_offset_from_range = 0;
        } else {
            self.next_offset_from_range += 1;
        }

        Some(T::get_refs(page, index_in_page as usize, &offsets))
    }
}

impl<'a, T: ComponentQueryAccess> EntityComponentQueryIter<'a, T> {
    pub fn with_entities(self) -> WithEntitiesIter<'a, T> {
        WithEntitiesIter { source_iter: self }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for WithEntitiesIter<'a, T> {
    type Item = (Entity, T::AccessOutput<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        self.source_iter.next().map(|components| unsafe {
            let entity = *self
                .source_iter
                .entities
                .get_unchecked(self.source_iter.current_entity_index);

            (entity, components)
        })
    }
}
