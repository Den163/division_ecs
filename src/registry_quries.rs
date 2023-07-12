use std::{marker::PhantomData, any::TypeId};

use crate::{Registry, Entity, archetype_data_page_view::ArchetypeDataPageView};

pub struct EntitiesReadQuery<'a, T0: 'static, T1: 'static> {
    page_views: Vec<ArchetypeDataPageView<'a>>,
    registry: &'a Registry,
    _types: (PhantomData<T0>, PhantomData<T1>)
}

pub struct EntitiesReadQueryIter<'a, T0: 'static, T1: 'static> {
    current_suitable_page_index: isize,
    current_entity_index: isize,
    page_views: &'a [ArchetypeDataPageView<'a>],
    entities_version: &'a [u32],
    page_entities_ids: &'a [u32],
    slices_buffer: (&'a [T0], &'a [T1]),
}

impl Registry {
    pub fn read_query<'a, T0: 'static, T1: 'static>(&'a self) -> EntitiesReadQuery<'a, T0, T1> {
        EntitiesReadQuery {
            page_views: Vec::new(),
            registry: self,
            _types: (PhantomData::default(), PhantomData::default())
        }
    }
}

impl<'a, T0: 'static, T1: 'static> Iterator for EntitiesReadQueryIter<'a, T0, T1> {
    type Item = (Entity, &'a T0, &'a T1);

    fn next(&mut self) -> Option<Self::Item> {
        let suitable_pages = &self.page_views;
        let last_entity_index = self.page_entities_ids.len() as isize - 1;

        if (self.current_suitable_page_index < 0) | 
           (self.current_entity_index >= last_entity_index) 
        {
            self.current_suitable_page_index += 1;
            self.current_entity_index = -1;

            if self.current_suitable_page_index >= suitable_pages.len() as isize {
                return None;
            }

            let page_view = &suitable_pages[self.current_suitable_page_index as usize];
            self.slices_buffer = (
                page_view.get_component_slice(),
                page_view.get_component_slice()
            );
            self.page_entities_ids = page_view.page.entities_ids();
        }

        self.current_entity_index += 1;

        let current_entity_index = self.current_entity_index as usize;
        let entity_id = self.page_entities_ids[current_entity_index];

        return Some((
            Entity {
                id: entity_id,
                version: self.entities_version[entity_id as usize]
            },
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
            .get_suitable_page_views(&include_types);

        self.page_views.clear();
        self.page_views.extend(suitable_indices_iter);

        EntitiesReadQueryIter {
            page_entities_ids: &[],
            slices_buffer: (&[], &[]),
            page_views: &self.page_views,
            entities_version: self.registry.entities_container.get_entity_versions(),
            current_suitable_page_index: -1,
            current_entity_index: -1,
        }
    }
}