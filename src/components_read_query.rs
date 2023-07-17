use std::{any::TypeId, marker::PhantomData};

use crate::{
    archetype_data_page_view::ArchetypeDataPageView, Entity, Registry, 
    TupleOfSliceToTupleOfElementRef
};

// TODO: Better divide non code-gen and generated data to reduce object file size

pub struct ComponentsReadQuery<'a, T> {
    page_views: Vec<ArchetypeDataPageView<'a>>,
    registry: &'a Registry,
    t: PhantomData<T>
}

pub struct ComponentsReadQueryIter<'a, TResult> {
    current_suitable_page_index: isize,
    current_entity_index: isize,
    page_views: &'a [ArchetypeDataPageView<'a>],
    entities_version: &'a [u32],
    page_entities_ids: &'a [u32],
    slices_buffer: TResult,
}

impl Registry {
    pub fn read_query<'a, TResult>(&'a self) -> ComponentsReadQuery<'a, TResult> {
        ComponentsReadQuery {
            page_views: Vec::new(),
            registry: self,
            t: PhantomData::<TResult>::default()
        }
    }
}

macro_rules! impl_entities_read_query {
    ($($T:tt),*) => {
        impl<'a, $($T: 'static,)*> Iterator for ComponentsReadQueryIter<'a, ($(&'a [$T],)*)> {
            type Item = (Entity, ($(&'a $T,)*));
        
            fn next(&mut self) -> Option<Self::Item> {
                let suitable_pages = &self.page_views;
                let last_entity_index = self.page_entities_ids.len() as isize - 1;
        
                if (self.current_suitable_page_index < 0) | (self.current_entity_index >= last_entity_index)
                {
                    self.current_suitable_page_index += 1;
                    self.current_entity_index = -1;
        
                    if self.current_suitable_page_index >= suitable_pages.len() as isize {
                        return None;
                    }
        
                    let page_view = &suitable_pages[self.current_suitable_page_index as usize];
                    self.slices_buffer = ($(page_view.get_component_slice::<$T>(),)*);
                    self.page_entities_ids = page_view.page.entities_ids();
                }
        
                self.current_entity_index += 1;
        
                let current_entity_index = self.current_entity_index as usize;
                let entity_id = self.page_entities_ids[current_entity_index];
                let slices_buffer = self.slices_buffer;
        
                return Some((
                    Entity {
                        id: entity_id,
                        version: self.entities_version[entity_id as usize],
                    },
                    slices_buffer.as_refs_tuple(current_entity_index)
                ));
            }
        }

        impl<'a, $($T : 'static,)*> IntoIterator for &'a mut ComponentsReadQuery<'a, ($($T,)*)> {
            type Item = (Entity, ($(&'a $T,)*));
            type IntoIter = ComponentsReadQueryIter<'a, ($(&'a [$T],)*)>;

            fn into_iter(self) -> Self::IntoIter {
                self.page_views.clear();

                {
                    let include_types = [$(TypeId::of::<$T>(),)*];
                    self.registry.archetypes_container
                        .fill_suitable_page_views(&include_types, &mut self.page_views);
                }

                ComponentsReadQueryIter {
                    page_entities_ids: &[],
                    slices_buffer: ($(&[] as &[$T],)*),
                    page_views: &self.page_views,
                    entities_version: self.registry.entities_container.get_entity_versions(),
                    current_suitable_page_index: -1,
                    current_entity_index: -1,
                }
            }
        }
    };
}

impl_entities_read_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_entities_read_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_entities_read_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_entities_read_query!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
impl_entities_read_query!(T0, T1, T2, T3, T4, T5, T6, T7);
impl_entities_read_query!(T0, T1, T2, T3, T4, T5, T6);
impl_entities_read_query!(T0, T1, T2, T3, T4, T5);
impl_entities_read_query!(T0, T1, T2, T3, T4);
impl_entities_read_query!(T0, T1, T2, T3);
impl_entities_read_query!(T0, T1, T2);
impl_entities_read_query!(T0, T1);
impl_entities_read_query!(T0);

