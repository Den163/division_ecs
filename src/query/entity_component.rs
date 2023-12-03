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
    filtered_entities: Vec<Entity>,
    entity_index_ranges: Vec<Range<usize>>,
    range_to_component_offsets: Vec<T::OffsetsTuple>,
    range_to_pages: Vec<*const ArchetypeDataPage>,
}

pub struct EntityComponentQueryIter<'a, T: ComponentQueryAccess> {
    store: &'a Store,
    entities: &'a [Entity],
    entities_chunks: &'a [Range<usize>],
    chunk_components_offsets: &'a [T::OffsetsTuple],
    chunk_pages: &'a [*const ArchetypeDataPage],
    current_entity_index: usize,

    next_chunk_index: usize,
    next_offset_from_chunk: usize,
}

pub struct WithEntitiesIter<'a, T: ComponentQueryAccess> {
    source_iter: EntityComponentQueryIter<'a, T>,
}

impl<T: ComponentQueryAccess> EntityComponentQuery<T> {
    pub fn new() -> Self {
        Self {
            filtered_entities: Vec::new(),
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
        entities: &[Entity],
        query: &'b mut EntityComponentQuery<T>,
    ) -> EntityComponentQueryIter<'a, T> {
        query.entity_index_ranges.clear();
        query.range_to_pages.clear();
        query.range_to_component_offsets.clear();
        query.filtered_entities.clear();

        query.filtered_entities.reserve(entities.len());

        // TODO: This filter need to optimize strongly
        query.filtered_entities.extend(
            entities
                .iter()
                .filter(|e| {
                    if let Some(arch) = self.get_entity_archetype(**e)  {
                        T::is_archetype_include_types(arch)
                    } else {
                        false
                    }
                } ),
        );

        let mut chunk_start = 0;
        while chunk_start < query.filtered_entities.len() {
            let chunk_entity = query.filtered_entities[chunk_start];
            let chunk_page_index =
                unsafe { self.get_page_index_unchecked(chunk_entity.id) as usize };
            let chunk_arch_index = self
                .archetypes_container
                .get_archetype_index_by_page(chunk_page_index);
            let chunk_arch =
                &self.archetypes_container.get_archetypes()[chunk_arch_index];
            let chunk_page = unsafe {
                self.archetypes_container
                    .get_pages()
                    .get_unchecked(chunk_page_index)
            };
            let chunk_comp_offsets = T::get_offsets(&chunk_arch);

            let mut chunk_end = chunk_start + 1;
            while chunk_end < query.filtered_entities.len() {
                let e = query.filtered_entities[chunk_end];
                let page_index = unsafe { self.get_page_index_unchecked(e.id) as usize };
                if page_index != chunk_page_index {
                    break;
                }

                chunk_end += 1;
            }

            query.entity_index_ranges.push(chunk_start..chunk_end);
            query.range_to_component_offsets.push(chunk_comp_offsets);
            query
                .range_to_pages
                .push(chunk_page as *const ArchetypeDataPage);

            chunk_start = chunk_end;
        }

        EntityComponentQueryIter {
            store: &self,
            entities: &query.filtered_entities,
            entities_chunks: &query.entity_index_ranges,
            chunk_pages: &query.range_to_pages,
            chunk_components_offsets: &query.range_to_component_offsets,
            next_chunk_index: 0,
            next_offset_from_chunk: 0,
            current_entity_index: 0,
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for EntityComponentQueryIter<'a, T> {
    type Item = T::AccessOutput<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_chunk_index >= self.entities_chunks.len() {
            return None;
        }

        let current_chunk =
            unsafe { self.entities_chunks.get_unchecked(self.next_chunk_index) };
        self.current_entity_index = current_chunk.start + self.next_offset_from_chunk;

        let (index_in_page, page, offsets) = unsafe {
            let current_entity = self.entities.get_unchecked(self.current_entity_index);
            let page = *self.chunk_pages.get_unchecked(self.next_chunk_index);
            (
                self.store.get_index_in_page_unchecked(current_entity.id),
                &*page,
                self.chunk_components_offsets
                    .get_unchecked(self.next_chunk_index),
            )
        };

        if self.current_entity_index >= current_chunk.end - 1 {
            self.next_chunk_index += 1;
            self.next_offset_from_chunk = 0;
        } else {
            self.next_offset_from_chunk += 1;
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
            (
                *self
                    .source_iter
                    .entities
                    .get_unchecked(self.source_iter.current_entity_index),
                components,
            )
        })
    }
}
