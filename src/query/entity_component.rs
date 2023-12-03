use std::ops::Range;

use crate::{
    archetype_data_page::ArchetypeDataPage,
    entity_in_archetype::EntityInArchetype,
    query::access::{ComponentQueryAccess, ReadWriteAccess, ReadonlyAccess, WriteAccess},
    tuple::ComponentsTuple,
    Entity, Store,
};

pub type EntityComponentReadWriteQuery<R, W> =
    EntityComponentQuery<ReadWriteAccess<R, W>>;
pub type EntityComponentReadOnlyQuery<R> = EntityComponentQuery<ReadonlyAccess<R>>;
pub type EntityComponentWriteQuery<W> = EntityComponentQuery<WriteAccess<W>>;

pub struct EntityComponentQuery<T: ComponentQueryAccess> {
    entities: Vec<Entity>,

    entity_index_ranges: Vec<Range<usize>>,
    range_to_component_offsets: Vec<T::OffsetsTuple>,
    range_to_pages: Vec<*const ArchetypeDataPage>,
}

pub struct EntityComponentQueryIter<'a, T: ComponentQueryAccess> {
    entities: &'a [Entity],
    entities_chunks: &'a [Range<usize>],
    chunk_components_offsets: &'a [T::OffsetsTuple],
    chunk_pages: &'a [*const ArchetypeDataPage],
    entity_in_archetypes: *const EntityInArchetype,
    current_entity_id: usize,

    next_chunk_index: usize,
    next_offset_from_chunk: usize,
}

pub struct WithEntitiesIter<'a, T: ComponentQueryAccess> {
    source_iter: EntityComponentQueryIter<'a, T>,
}

impl<T: ComponentQueryAccess> EntityComponentQuery<T> {
    pub fn new() -> Self {
        Self::for_entities(&[])
    }

    pub fn for_entities(entities: &[Entity]) -> Self {
        EntityComponentQuery {
            entities: Vec::from(entities),
            entity_index_ranges: Vec::new(),
            range_to_component_offsets: Vec::new(),
            range_to_pages: Vec::new(),
        }
    }

    pub fn set_entities(&mut self, entities: &[Entity]) {
        self.entities.clear();
        self.entities.extend_from_slice(entities);
    }
}

pub fn readonly<R: ComponentsTuple>(
    entities: &[Entity],
) -> EntityComponentReadOnlyQuery<R> {
    EntityComponentQuery::for_entities(entities)
}

pub fn write<W: ComponentsTuple>(entities: &[Entity]) -> EntityComponentWriteQuery<W> {
    EntityComponentQuery::for_entities(entities)
}

pub fn read_write<R: ComponentsTuple, W: ComponentsTuple>(
    entities: &[Entity],
) -> EntityComponentReadWriteQuery<R, W> {
    EntityComponentQuery::for_entities(entities)
}

impl Store {
    pub fn entity_component_query_iter<'a, 'b: 'a, T: ComponentQueryAccess>(
        &'a self,
        query: &'b mut EntityComponentQuery<T>,
    ) -> EntityComponentQueryIter<'a, T> {
        query.entity_index_ranges.clear();
        query.range_to_pages.clear();
        query.range_to_component_offsets.clear();

        let entity_in_archetypes = self.entity_in_archetypes();

        let mut i = 0;
        while i < query.entities.len() {
            let chunk_entity = query.entities[i];
            let chunk_page_location =
                unsafe { *entity_in_archetypes.add(chunk_entity.id as usize) };
            let chunk_page_index = chunk_page_location.page_index as usize;
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

            let mut j = i + 1;
            while j < query.entities.len() {
                let e = query.entities[j];
                let page_location = unsafe { *entity_in_archetypes.add(e.id as usize) };

                if page_location.page_index != chunk_page_location.page_index {
                    break;
                }

                j += 1;
            }

            query.entity_index_ranges.push(i..j);
            query.range_to_component_offsets.push(chunk_comp_offsets);
            query
                .range_to_pages
                .push(chunk_page as *const ArchetypeDataPage);

            i = j;
        }

        EntityComponentQueryIter {
            entities: &query.entities,
            entities_chunks: &query.entity_index_ranges,
            chunk_pages: &query.range_to_pages,
            chunk_components_offsets: &query.range_to_component_offsets,
            entity_in_archetypes: self.entity_in_archetypes(),
            next_chunk_index: 0,
            next_offset_from_chunk: 0,
            current_entity_id: 0,
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
        self.current_entity_id = current_chunk.start + self.next_offset_from_chunk;

        let (entity_in_archetype, page, offsets) = unsafe {
            let current_entity = self.entities.get_unchecked(self.current_entity_id);
            let entity_in_archetype =
                *self.entity_in_archetypes.add(current_entity.id as usize);
            let page = *self.chunk_pages.get_unchecked(self.next_chunk_index);
            (
                entity_in_archetype,
                &*page,
                self.chunk_components_offsets
                    .get_unchecked(self.next_chunk_index),
            )
        };

        if self.current_entity_id >= current_chunk.end - 1 {
            self.next_chunk_index += 1;
            self.next_offset_from_chunk = 0;
        } else {
            self.next_offset_from_chunk += 1;
        }

        Some(T::get_refs(
            page,
            entity_in_archetype.index_in_page as usize,
            &offsets,
        ))
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
                    .get_unchecked(self.source_iter.current_entity_id),
                components,
            )
        })
    }
}
