use std::marker::PhantomData;

use crate::{
    Entity, Registry, archetype_data_page::ArchetypeDataPage, tuple::{ComponentsTuple, ComponentsRefsTuple}
};

pub trait QueryIterator<T> where T: ComponentsTuple {
    fn iter<'a, 'b: 'a>(&'a self, query: &'b mut ComponentsReadQuery<T>) -> ComponentsReadQueryIter<'a, T>;
}

pub struct ComponentsReadQuery<T> where T: ComponentsTuple {
    page_views: Vec<PageIterView>,
    components_offsets: Vec<T::OffsetsTuple>,
    _phantom_: PhantomData<T>
}

pub struct ComponentsReadQueryIter<'a, T> where T: ComponentsTuple {
    page_views: &'a [PageIterView],
    components_offsets: &'a [T::OffsetsTuple],
    entities_versions: &'a [u32],

    current_page_view_index: usize,
    current_entity_index: usize,

    _phantom_: PhantomData<T>
}

struct PageIterView {
    page: *const ArchetypeDataPage,
    components_offsets_index: usize
}

impl Registry {
    pub fn read_query<T>(&self) -> ComponentsReadQuery<T> where T: ComponentsTuple {
        ComponentsReadQuery {
            page_views: Vec::new(),
            components_offsets: Vec::new(),
            _phantom_: PhantomData::<T>::default()
        }
    }
}

impl<T> QueryIterator<T> for Registry where T: ComponentsTuple {
    fn iter<'a, 'b: 'a>(&'a self, query: &'b mut ComponentsReadQuery<T>) -> ComponentsReadQueryIter<'a, T> {
        let arch_container = &self.archetypes_container;
        let archetypes = arch_container.get_archetypes();
        let layouts = arch_container.get_layouts();
        let pages = arch_container.get_pages();

        query.components_offsets.clear();
        query.page_views.clear();

        for (arch_idx, arch) in archetypes.into_iter().enumerate() {
            if T::is_archetype_include_types(arch) == false {
                continue;
            }

            let offsets = layouts[arch_idx].component_offsets();
            query.components_offsets.push(T::get_offsets(arch, offsets));

            let components_offsets_index = query.components_offsets.len() - 1;
            let arch_pages = arch_container.get_archetype_page_indices(arch_idx);

            for page_idx in arch_pages {
                let page = &pages[*page_idx];
                if page.entities_count() == 0 {
                    continue;
                }

                query.page_views.push(PageIterView { page, components_offsets_index });
            }
        }

        ComponentsReadQueryIter {
            _phantom_: PhantomData::default(),
            current_page_view_index: 0,
            current_entity_index: 0,
            components_offsets: &query.components_offsets,
            page_views: &query.page_views,
            entities_versions: self.entities_container.get_entity_versions(),
        }
    }
}

macro_rules! components_read_query_impl {
    ($($T:tt),*) => {
        impl<'a, $($T: 'static),*> Iterator for ComponentsReadQueryIter<'a, ($($T,)*)> {
            type Item = (Entity, ($(&'a $T,)*));
            
            #[inline(always)]
            fn next(&mut self) -> Option<Self::Item> {
                let page_views = self.page_views;
                let page_view_count = self.page_views.len();
        
                if self.current_page_view_index >= page_view_count {
                    return None;
                }
        
                let mut curr_page_view = &page_views[self.current_page_view_index];
                let mut curr_page = curr_page_view.page;
                let mut entities_ids = unsafe { (&*curr_page).entities_ids() };
        
                if self.current_entity_index >= entities_ids.len() {
                    self.current_page_view_index += 1;
        
                    if self.current_page_view_index >= page_view_count {
                        return None;
                    }
        
                    self.current_entity_index = 0;
        
                    curr_page_view = unsafe { page_views.get_unchecked(self.current_page_view_index) };
                    curr_page = curr_page_view.page;
                    entities_ids = unsafe { (&*curr_page).entities_ids() };
                }
                
                unsafe {
                    let curr_entity_idx = self.current_entity_index;
                    let id = *entities_ids.get_unchecked(curr_entity_idx);
                    let version = *self.entities_versions.get_unchecked(id as usize);
                    let offsets = &self.components_offsets.get_unchecked(curr_page_view.components_offsets_index);
        
                    self.current_entity_index += 1;
        
                    return Some((
                        Entity { id, version },
                        <($($T,)*) as ComponentsRefsTuple<'a, ($($T,)*)>>::get_refs(&*curr_page, curr_entity_idx, offsets)
                    ));
                }
            }
        }        
    };
}

components_read_query_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
components_read_query_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
components_read_query_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
components_read_query_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
components_read_query_impl!(T0, T1, T2, T3, T4, T5, T6, T7);
components_read_query_impl!(T0, T1, T2, T3, T4, T5, T6);
components_read_query_impl!(T0, T1, T2, T3, T4, T5);
components_read_query_impl!(T0, T1, T2, T3, T4);
components_read_query_impl!(T0, T1, T2, T3);
components_read_query_impl!(T0, T1, T2);
components_read_query_impl!(T0, T1);
components_read_query_impl!(T0);