use std::ops::Range;

use crate::{
    component_query_access::ComponentQueryAccess,
    entity_in_archetype::EntityInArchetype, Entity, Store, archetype_data_page::ArchetypeDataPage,
};

pub type EntityComponentReadWriteQuery<TRead, TWrite> = EntityComponentQuery<(TRead, TWrite)>;
pub type EntityComponentReadOnlyQuery<TRead> = EntityComponentQuery<(TRead, ())>;
pub type EntityComponentWriteQuery<TWrite> = EntityComponentQuery<((), TWrite)>;

pub struct EntityComponentQuery<T: ComponentQueryAccess> {
    entities: Vec<Entity>,
    entities_chunks: Vec<Range<usize>>,
    chunk_component_offsets: Vec<T::OffsetsTuple>,
    chunk_pages: Vec<*const ArchetypeDataPage>,
}

pub struct EntityComponentQueryIter<'a, T: ComponentQueryAccess> {
    entities: &'a [Entity],
    entities_chunks: &'a [Range<usize>],
    chunk_components_offsets: &'a [T::OffsetsTuple],
    chunk_pages: &'a [*const ArchetypeDataPage],
    entity_in_archetypes: *const EntityInArchetype,

    current_chunk_index: usize,
    current_offset_from_chunk: usize,
}

impl<T: ComponentQueryAccess> EntityComponentQuery<T> {
    pub fn with_entities(entities: &[Entity]) -> Self {
        EntityComponentQuery {
            entities: Vec::from(entities),
            chunk_component_offsets: Vec::new(),
            entities_chunks: Vec::new(),
            chunk_pages: Vec::new(),
        }
    }

    pub fn set_entities(&mut self, entities: &[Entity]) {
        self.entities.clear();
        self.entities.extend_from_slice(entities);
    }
}

impl Store {
    pub fn entity_component_query_iter<'a, 'b: 'a, T: ComponentQueryAccess>(
        &'a self,
        query: &'b mut EntityComponentQuery<T>,
    ) -> EntityComponentQueryIter<'a, T> {
        query.entities_chunks.clear();
        query.chunk_pages.clear();

        let entity_in_archetypes = self.entity_in_archetypes();

        let mut i = 0;
        while i < query.entities.len() {
            let chunk_entity = query.entities[i];
            let chunk_page_location =
                unsafe { *entity_in_archetypes.add(chunk_entity.id as usize) };
            let chunk_page_index = chunk_page_location.page_index as usize;
            let chunk_arch_index = self.archetypes_container.get_archetype_index_by_page(
                chunk_page_index);
            let chunk_arch = &self.archetypes_container.get_archetypes()[chunk_arch_index];
            let chunk_page = unsafe {
                self.archetypes_container.get_pages().get_unchecked(chunk_page_index) 
            };
            let chunk_comp_offsets = T::get_offsets(&chunk_arch);

            let mut j = i + 1;
            while j < query.entities.len() {
                let e = query.entities[j];
                let page_location = unsafe { 
                    *entity_in_archetypes.add(e.id as usize) 
                };

                if page_location.page_index != chunk_page_location.page_index {
                    break;
                }

                j+= 1;
            }

            query.entities_chunks.push(i..j);
            query.chunk_component_offsets.push(chunk_comp_offsets);
            query.chunk_pages.push(chunk_page as *const ArchetypeDataPage);

            i+= j;
        }

        EntityComponentQueryIter {
            entities: &query.entities,
            entities_chunks: &query.entities_chunks,
            chunk_pages: &query.chunk_pages,
            chunk_components_offsets: &query.chunk_component_offsets,
            entity_in_archetypes: self.entity_in_archetypes(),
            current_chunk_index: 0,
            current_offset_from_chunk: 0,
        }
    }
}

impl<'a, T: ComponentQueryAccess> Iterator for EntityComponentQueryIter<'a, T> {
    type Item = (Entity, T::AccessOutput<'a>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_chunk_index >= self.entities_chunks.len() {
            return None;
        }

        let current_chunk = unsafe {
            self.entities_chunks.get_unchecked(self.current_chunk_index)
        };
        let current_entity_index = current_chunk.start + self.current_offset_from_chunk;

        let (current_entity, entity_in_archetype, page, offsets) = unsafe {
            let current_entity = *self.entities.get_unchecked(current_entity_index);
            let entity_in_archetype =
                *self.entity_in_archetypes.add(current_entity.id as usize);
            let page = *self.chunk_pages.get_unchecked(self.current_chunk_index);
            (
                current_entity,
                entity_in_archetype,
                &*page,
                self.chunk_components_offsets.get_unchecked(self.current_chunk_index),
            )
        };

        if current_entity_index >= current_chunk.end - 1 {
            self.current_chunk_index += 1;
            self.current_offset_from_chunk = 0;
        } else {
            self.current_offset_from_chunk += 1;
        }

        Some((
            current_entity,
            T::get_refs(page, entity_in_archetype.index_in_page as usize, &offsets),
        ))
    }
}
