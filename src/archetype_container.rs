use crate::{
    archetype::Archetype, 
    archetype_data_layout::ArchetypeDataLayout,
    archetype_data_page::ArchetypeDataPage, mem_utils,
};

pub struct ArchetypeToPages {
    pages: Vec<usize>
}

pub(crate) struct ArchetypeContainer {
    archetypes: *mut Archetype,
    archetype_to_pages: *mut ArchetypeToPages,
    archetype_layouts: *mut ArchetypeDataLayout,
    archetype_count: usize,

    pages: *mut ArchetypeDataPage,
    page_count: usize,

    free_archetypes: Vec<usize>,
    free_pages: Vec<usize>,
}

impl ArchetypeContainer {
    const ARCHETYPE_DEFAULT_CAPACITY: usize = 5;
    const ARCHETYPE_PAGE_DEFAULT_CAPACITY: usize = 5;

    pub fn new() -> ArchetypeContainer {
        let archetype_count = Self::ARCHETYPE_DEFAULT_CAPACITY;
        let archetypes = mem_utils::alloc(archetype_count);
        let archetype_layouts = mem_utils::alloc(archetype_count);
        let archetype_to_pages = mem_utils::alloc(archetype_count);

        let page_count = Self::ARCHETYPE_PAGE_DEFAULT_CAPACITY;
        let pages = mem_utils::alloc(page_count);

        let free_archetypes = (0..archetype_count).collect();
        let free_pages = (0..page_count).collect();

        ArchetypeContainer {
            archetypes,
            archetype_layouts,
            archetype_to_pages,
            archetype_count,

            pages,
            page_count,

            free_archetypes,
            free_pages
        }
    }
}
