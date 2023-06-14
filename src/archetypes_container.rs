use crate::{
    archetype::Archetype, archetype_data_layout::ArchetypeDataLayout,
    archetype_data_page::ArchetypeDataPage, entity_in_archetype::EntityInArchetype,
};

#[derive(Debug)]
pub(crate) struct ArchetypesContainer {
    archetypes: Vec<Archetype>,
    archetype_to_pages: Vec<ArchetypePages>,
    archetype_layouts: Vec<ArchetypeDataLayout>,
    pages: Vec<ArchetypeDataPage>,

    free_archetypes: Vec<usize>,
    free_pages: Vec<usize>,
}

#[derive(Debug)]
struct ArchetypePages {
    pages: Vec<usize>,
}

impl ArchetypesContainer {
    const ARCHETYPE_DEFAULT_CAPACITY: usize = 5;
    const ARCHETYPE_PAGE_DEFAULT_CAPACITY: usize = 5;

    pub fn new() -> ArchetypesContainer {
        let archetypes = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);
        let archetype_to_pages = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);
        let archetype_layouts = Vec::with_capacity(Self::ARCHETYPE_DEFAULT_CAPACITY);
        let free_archetypes = Vec::new();

        let pages = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY)
            .map(|_| ArchetypeDataPage::new())
            .collect();

        let free_pages = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY).collect();

        ArchetypesContainer {
            archetypes,
            archetype_to_pages,
            archetype_layouts,
            pages,

            free_archetypes,
            free_pages,
        }
    }

    pub fn add_entity(&mut self, entity_id: u32, archetype: &Archetype) -> EntityInArchetype {
        let archetype_index = self.reserve_archetype(archetype);
        let pages = &mut self.archetype_to_pages[archetype_index].pages;

        for page_index in pages {
            let page_index = *page_index;
            let page = &mut self.pages[page_index];
            if page.has_free_space() {
                page.push_entity_id(entity_id);

                return EntityInArchetype {
                    archetype_index,
                    page_index,
                };
            }
        }

        let page_index = self.reserve_page_for_archetype(archetype_index);
        self.pages[page_index].push_entity_id(entity_id);

        EntityInArchetype {
            archetype_index,
            page_index,
        }
    }

    pub fn remove_entity(&mut self, entity_id: u32, entity_in_archetype: EntityInArchetype) {
        let page = &mut self.pages[entity_in_archetype.page_index];

        page.remove_entity_id(entity_id);
    }

    pub fn get_component_ref<'a, T: 'static>(
        &'a self,
        entity_id: u32,
        entity_in_archetype: EntityInArchetype,
    ) -> &T {
        let component_offset = self.get_valid_component_offset::<T>(entity_in_archetype.archetype_index);
        let page = &self.pages[entity_in_archetype.page_index];

        page.get_component_ref(component_offset, page.get_entity_index(entity_id))
    }

    pub fn get_component_ref_mut<'a, T: 'static>(
        &'a mut self,
        entity_id: u32,
        entity_in_archetype: EntityInArchetype,
    ) -> &'a mut T {
        let component_offset = self.get_valid_component_offset::<T>(entity_in_archetype.archetype_index);
        let page = &mut self.pages[entity_in_archetype.page_index];

        page.get_component_ref_mut(component_offset, page.get_entity_index(entity_id))
    }

    pub fn get_valid_component_offset<T: 'static>(&self, archetype_index: usize) -> usize {
        let archetype = &self.archetypes[archetype_index];
        let layout = &self.archetype_layouts[archetype_index];

        let component_index = archetype.find_component_index(std::any::TypeId::of::<T>());
        debug_assert!(
            component_index.is_some(), 
            "There is no component {} in the given archetype", std::any::type_name::<T>()
        );

        unsafe {
            let component_index = component_index.unwrap_unchecked();
            *layout.component_offsets().add(component_index)
        }
    }

    pub fn get_include_archetypes(&self, archetype: &Archetype) -> Vec<usize> {
        self.archetypes.iter().enumerate()
            .filter(|(i, arch)| { archetype.is_include(arch) })
            .map(|(i, _)| { i })
            .collect()
    }

    pub fn get_indexed_component_ref<'a, T: 'static>(
        &'a self, page_index: usize, component_offset: usize, entity_index: usize
    ) -> &'a T {
        self.pages[page_index].get_component_ref(component_offset, entity_index)
    }

    pub fn get_indexed_component_ref_mut<'a, T: 'static>(
        &'a mut self, page_index: usize, component_offset: usize, entity_index: usize
    ) -> &'a T {
        self.pages[page_index].get_component_ref_mut(component_offset, entity_index)
    } 

    fn reserve_archetype(&mut self, archetype: &Archetype) -> usize {
        for (i, arch) in self.archetypes.iter().enumerate() {
            if arch.is_same_as(&archetype) {
                return i;
            }
        }

        match self.free_archetypes.pop() {
            Some(free_idx) => {
                self.archetype_layouts[free_idx] = ArchetypeDataLayout::new(&archetype);
                self.archetypes[free_idx] = archetype.clone();

                free_idx
            }
            None => {
                let new_size = self.archetypes.len() + 1;
                self.archetype_layouts
                    .resize_with(new_size, || ArchetypeDataLayout::new(&archetype));
                self.archetypes.resize_with(new_size, || archetype.clone());
                self.archetype_to_pages
                    .resize_with(new_size, || ArchetypePages { pages: Vec::new() });

                new_size - 1
            }
        }
    }

    fn reserve_page_for_archetype(&mut self, archetype_index: usize) -> usize {
        let page_index = match self.free_pages.pop() {
            Some(page_index) => page_index,
            None => {
                let page_index = self.pages.len();
                self.pages.insert(page_index, ArchetypeDataPage::new());
                page_index
            }
        };

        let layout = &self.archetype_layouts[archetype_index];
        self.pages[page_index].set_layout(layout);

        self.archetype_to_pages[archetype_index]
            .pages
            .push(page_index);
        page_index
    }
}
