use std::{marker::PhantomData, any::TypeId};

use crate::{Registry, Entity};

pub struct EntitiesReadQuery<'a, T0: 'static, T1: 'static> {
    suitable_pages_indices_buffer: Vec<usize>,
    registry: &'a Registry,
    _types: (PhantomData<T0>, PhantomData<T1>)
}

pub struct EntitiesReadQueryIter<'a, T0: 'static, T1: 'static> {
    current_suitable_page_index: isize,
    current_entity_index: isize,
    read_query: &'a EntitiesReadQuery<'a, T0, T1>,
    entity_ids: &'a [u32],
    slices_buffer: (&'a [T0], &'a [T1]),
}

impl Registry {
    pub fn read_query<'a, T0: 'static, T1: 'static>(&'a self) -> EntitiesReadQuery<'a, T0, T1> {
        EntitiesReadQuery {
            suitable_pages_indices_buffer: Vec::new(),
            registry: self,
            _types: (PhantomData::default(), PhantomData::default())
        }
    }
}

impl<'a, T0: 'static, T1: 'static> Iterator for EntitiesReadQueryIter<'a, T0, T1> {
    type Item = (Entity, &'a T0, &'a T1);

    fn next(&mut self) -> Option<Self::Item> {
        let suitable_pages = &self.read_query.suitable_pages_indices_buffer;
        let last_entity_index = self.entity_ids.len() as isize - 1;

        if (self.current_suitable_page_index < 0) | 
           (self.current_entity_index >= last_entity_index) 
        {
            self.current_suitable_page_index += 1;
            self.current_entity_index = -1;

            if self.current_suitable_page_index >= suitable_pages.len() as isize {
                return None;
            }

            let page_index = suitable_pages[self.current_suitable_page_index as usize];
            let archetype_container = &self.read_query.registry.archetypes_container;
            self.slices_buffer = archetype_container.get_components_slices::<T0, T1>(page_index);
            self.entity_ids = archetype_container.get_page_entity_ids_slice(
                page_index);
        }

        self.current_entity_index += 1;

        let current_entity_index = self.current_entity_index as usize;
        let entity_id = self.entity_ids[current_entity_index];

        return Some((
            self.read_query.registry.entities_container.get_entity_by_id(entity_id),
            &self.slices_buffer.0[current_entity_index],
            &self.slices_buffer.1[current_entity_index],
        ))
    }
}

impl<'a, T0: 'static, T1: 'static> IntoIterator for &'a mut EntitiesReadQuery<'a, T0, T1> {
    type Item = (Entity, &'a T0, &'a T1);
    type IntoIter = EntitiesReadQueryIter<'a, T0, T1>;

    fn into_iter(self) -> Self::IntoIter {
        let include_types = [ TypeId::of::<T0>(), TypeId::of::<T1>(), ];
        let suitable_indices_iter = self.registry.archetypes_container
            .get_suitable_page_indices(&include_types);

        self.suitable_pages_indices_buffer.clear();
        self.suitable_pages_indices_buffer.extend(suitable_indices_iter);

        EntitiesReadQueryIter {
            entity_ids: &[],
            slices_buffer: (&[], &[]),
            read_query: self,
            current_suitable_page_index: -1,
            current_entity_index: -1,
        }
    }
}