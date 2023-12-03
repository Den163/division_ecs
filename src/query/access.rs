use std::marker::PhantomData;

use crate::{archetype_data_page::ArchetypeDataPage, tuple::ComponentsTuple, Archetype};

pub struct ReadonlyAccess<TRead: ComponentsTuple> {
    _read: PhantomData<TRead>,
}

pub struct WriteAccess<TWrite: ComponentsTuple> {
    _write: PhantomData<TWrite>,
}

pub struct ReadWriteAccess<TRead: ComponentsTuple, TWrite: ComponentsTuple> {
    _read: PhantomData<TRead>,
    _write: PhantomData<TWrite>,
}

pub trait ComponentQueryAccess {
    type OffsetsTuple;
    type AccessOutput<'a>;

    fn is_archetype_include_types(archetype: &Archetype) -> bool;

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetsTuple,
    ) -> Self::AccessOutput<'a>;

    fn get_offsets(archetype: &Archetype) -> Self::OffsetsTuple;
}

impl<TRead, TWrite> ComponentQueryAccess for ReadWriteAccess<TRead, TWrite>
where
    TRead: ComponentsTuple,
    TWrite: ComponentsTuple,
{
    type OffsetsTuple = (TRead::OffsetsTuple, TWrite::OffsetsTuple);
    type AccessOutput<'a> = (TRead::RefsTuple<'a>, TWrite::MutRefsTuple<'a>);

    fn get_offsets(archetype: &Archetype) -> Self::OffsetsTuple {
        (
            TRead::get_offsets_unchecked(archetype),
            TWrite::get_offsets_unchecked(archetype),
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

impl<TRead: ComponentsTuple> ComponentQueryAccess for ReadonlyAccess<TRead> {
    type OffsetsTuple = TRead::OffsetsTuple;
    type AccessOutput<'a> = TRead::RefsTuple<'a>;

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

    fn get_offsets(archetype: &Archetype) -> Self::OffsetsTuple {
        TRead::get_offsets_unchecked(archetype)
    }
}

impl<TWrite: ComponentsTuple> ComponentQueryAccess for WriteAccess<TWrite> {
    type OffsetsTuple = TWrite::OffsetsTuple;
    type AccessOutput<'a> = TWrite::MutRefsTuple<'a>;

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

    fn get_offsets(archetype: &Archetype) -> Self::OffsetsTuple {
        TWrite::get_offsets_unchecked(archetype)
    }
}
