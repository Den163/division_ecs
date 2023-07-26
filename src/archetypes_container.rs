use crate::{
    archetype::Archetype, archetype_data_layout::ArchetypeDataLayout,
    archetype_data_page::ArchetypeDataPage, entity_in_archetype::EntityInArchetype, archetype_data_page_view::ArchetypeDataPageView,
};

#[derive(Debug)]
pub(crate) struct ArchetypesContainer {
    archetypes: Vec<Archetype>,
    archetype_to_pages: Vec<ArchetypePages>,
    archetype_layouts: Vec<ArchetypeDataLayout>,

    pages: Vec<ArchetypeDataPage>,
    page_to_archetype: Vec<isize>,

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

        let pages = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY)
            .map(|_| ArchetypeDataPage::new())
            .collect();

        let page_to_archetype = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY)
            .map(|_| -1)
            .collect();

        let free_archetypes = Vec::new();
        let free_pages = (0..Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY).collect();

        ArchetypesContainer {
            archetypes,
            archetype_to_pages,
            archetype_layouts,

            pages,
            page_to_archetype,

            free_archetypes,
            free_pages,
        }
    }

    pub fn add_entity(&mut self, entity_id: u32, archetype: &Archetype) -> EntityInArchetype {
        let archetype_index = self.reserve_archetype(archetype);
        let pages = &mut self.archetype_to_pages[archetype_index].pages;

        // Looking in reverse direction, because there is more probability, that
        // page with free slots will be located in the end
        for page_index in pages.into_iter().rev() {
            let page_index = *page_index;
            let page = &mut self.pages[page_index];
            if page.has_free_space() {
                page.push_entity_id(entity_id);

                return EntityInArchetype {
                    page_index,
                };
            }
        }

        let page_index = self.reserve_page_for_archetype(archetype_index);
        self.pages[page_index].push_entity_id(entity_id);

        EntityInArchetype {
            page_index,
        }
    }

    pub fn remove_entity(&mut self, entity_id: u32, entity_in_archetype: EntityInArchetype) {
        let page = &mut self.pages[entity_in_archetype.page_index];

        page.remove_entity_id(entity_id);
    }

    pub fn get_page_view(
        &self, 
        page_index: usize
    ) -> ArchetypeDataPageView {
        let arch_idx = self.page_to_archetype[page_index] as usize;
        ArchetypeDataPageView {
            archetype: &self.archetypes[arch_idx],
            layout: &self.archetype_layouts[arch_idx],
            page: &self.pages[page_index]
        }
    }

    #[inline]
    pub fn get_archetypes(&self) -> &[Archetype] {
        &self.archetypes
    }

    #[inline]
    pub fn get_layouts(&self) -> &[ArchetypeDataLayout] {
        &self.archetype_layouts
    }

    #[inline]
    pub fn get_pages(&self) -> &[ArchetypeDataPage] {
        &self.pages
    }

    #[inline]
    pub fn get_archetype_page_indices(&self, archetype_index: usize) -> &[usize] {
        &self.archetype_to_pages[archetype_index].pages
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
                self.page_to_archetype.insert(page_index, archetype_index as isize);
                page_index
            }
        };

        let layout = &self.archetype_layouts[archetype_index];
        self.pages[page_index].set_layout(layout);

        self.archetype_to_pages[archetype_index].pages.push(page_index);
        self.page_to_archetype[page_index] = archetype_index as isize;

        page_index
    }
}