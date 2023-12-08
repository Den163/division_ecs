use std::marker::PhantomData;

use crate::{archetype_data_page::ArchetypeDataPage, component_tuple::ComponentTuple, Archetype, archetype_layout::ArchetypeLayout};

pub struct ReadonlyAccess<TRead: ComponentTuple> {
    _read: PhantomData<TRead>,
}

pub struct WriteAccess<TWrite: ComponentTuple> {
    _write: PhantomData<TWrite>,
}

pub struct ReadWriteAccess<TRead: ComponentTuple, TWrite: ComponentTuple> {
    _read: PhantomData<TRead>,
    _write: PhantomData<TWrite>,
}

pub trait ComponentQueryAccess {
    type OffsetsTuple: Default + Copy;
    type AccessOutput<'a>;

    fn is_archetype_include_types(archetype: &Archetype) -> bool;

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a>;

    fn get_offsets(archetype: &Archetype, layout: &ArchetypeLayout) -> Self::OffsetsTuple;
}

impl<TRead, TWrite> ComponentQueryAccess for ReadWriteAccess<TRead, TWrite>
where
    TRead: ComponentTuple,
    TWrite: ComponentTuple,
{
    type OffsetsTuple = (TRead::OffsetTuple, TWrite::OffsetTuple);
    type AccessOutput<'a> = (TRead::RefTuple<'a>, TWrite::MutRefTuple<'a>);

    fn get_offsets(archetype: &Archetype, layout: &ArchetypeLayout) -> Self::OffsetsTuple {
        (
            TRead::get_offsets_unchecked(archetype, layout),
            TWrite::get_offsets_unchecked(archetype, layout),
        )
    }

    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TRead::is_archetype_include_types(archetype)
            && TWrite::is_archetype_include_types(archetype)
    }

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        (read_offsets, write_offsets): &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a> {
        (
            TRead::get_refs(page, entity_index, read_offsets),
            TWrite::get_refs_mut(page, entity_index, write_offsets),
        )
    }
}

impl<TRead: ComponentTuple> ComponentQueryAccess for ReadonlyAccess<TRead> {
    type OffsetsTuple = TRead::OffsetTuple;
    type AccessOutput<'a> = TRead::RefTuple<'a>;

    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TRead::is_archetype_include_types(archetype)
    }

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a> {
        TRead::get_refs(page, entity_index, offsets)
    }

    fn get_offsets(archetype: &Archetype, layout: &ArchetypeLayout) -> Self::OffsetsTuple {
        TRead::get_offsets_unchecked(archetype, layout)
    }
}

impl<TWrite: ComponentTuple> ComponentQueryAccess for WriteAccess<TWrite> {
    type OffsetsTuple = TWrite::OffsetTuple;
    type AccessOutput<'a> = TWrite::MutRefTuple<'a>;

    fn is_archetype_include_types(archetype: &Archetype) -> bool {
        TWrite::is_archetype_include_types(archetype)
    }

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a> {
        TWrite::get_refs_mut(page, entity_index, offsets)
    }

    fn get_offsets(archetype: &Archetype, layout: &ArchetypeLayout) -> Self::OffsetsTuple {
        TWrite::get_offsets_unchecked(archetype, layout)
    }
}
