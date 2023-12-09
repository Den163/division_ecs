use paste::paste;

use crate::{
    archetype_data_page::ArchetypeDataPage, type_ids, Archetype, ArchetypeBuilder,
    Component, archetype_layout::ArchetypeLayout,
};

pub trait ComponentTuple {
    type OffsetTuple: Default + Copy;
    type PtrTuple<'a>;
    type MutPtrTuple<'a>;
    type RefTuple<'a>;
    type MutRefTuple<'a>;

    fn get_offsets_unchecked(
        archetype: &Archetype,
        layout: &ArchetypeLayout,
    ) -> Self::OffsetTuple;

    fn get_offsets(
        archetype: &Archetype,
        layout: &ArchetypeLayout,
    ) -> Option<Self::OffsetTuple>;

    fn get_refs<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetTuple,
    ) -> Self::RefTuple<'a>;

    fn get_refs_mut<'a>(
        page: &'a ArchetypeDataPage,
        entity_index: usize,
        offsets: &Self::OffsetTuple,
    ) -> Self::MutRefTuple<'a>;

    fn get_ptrs<'a>(
        page: &'a ArchetypeDataPage, offsets: &Self::OffsetTuple
    ) -> Self::PtrTuple<'a>;

    fn get_ptrs_mut<'a>(
        page: &'a ArchetypeDataPage, offsets: &Self::OffsetTuple
    ) -> Self::MutPtrTuple<'a>;

    fn add_to_ptrs<'a>(ptrs: &Self::PtrTuple<'a>, entity_index: usize) -> Self::PtrTuple<'a>;
    fn add_to_ptrs_mut<'a>(ptrs: &Self::MutPtrTuple<'a>, entity_index: usize) -> Self::MutPtrTuple<'a>;

    fn null_ptrs<'a>() -> Self::PtrTuple<'a>;
    fn null_ptrs_mut<'a>() -> Self::MutPtrTuple<'a>;

    fn ptrs_to_refs<'a>(ptrs: Self::PtrTuple<'a>) -> Self::RefTuple<'a>;
    fn ptrs_to_refs_mut<'a>(ptrs: Self::MutPtrTuple<'a>) -> Self::MutRefTuple<'a>;

    fn assign_to_refs<'a>(refs: Self::MutRefTuple<'a>, values: Self);

    fn into_archetype() -> Archetype;
    fn is_archetype_include_types(archetype: &Archetype) -> bool;

    fn add_components_to_archetype_builder(
        builder: &mut ArchetypeBuilder,
    ) -> &mut ArchetypeBuilder;

    fn remove_components_from_archetype_builder(
        builder: &mut ArchetypeBuilder,
    ) -> &mut ArchetypeBuilder;
}

macro_rules! components_tuple_impl {
    ($($T:ident),*) => {
        #[allow(unused_parens)]
        impl<$($T: 'static + Component),*> ComponentTuple for ($($T),*) {
            type OffsetTuple = ($(components_tuple_impl!(@type_to_usize, $T)),*);
            type PtrTuple<'a> = ($(*const $T),*);
            type MutPtrTuple<'a> = ($(*mut $T),*);
            type RefTuple<'a> = ($(&'a $T),*);
            type MutRefTuple<'a> = ($(&'a mut $T),*);

            #[inline(always)]
            fn get_offsets_unchecked(
                archetype: &Archetype, layout: &ArchetypeLayout
            ) -> Self::OffsetTuple {
                unsafe {(
                    $(
                        *(
                            layout.component_offsets().add(
                                archetype.find_component_index_of::<$T>()
                                        .unwrap_unchecked()
                            )
                        )
                    ),*
                )}
            }

            #[inline(always)]
            fn get_offsets(
                archetype: &Archetype, layout: &ArchetypeLayout
            ) -> Option<Self::OffsetTuple> {
                unsafe {Some((
                    $({
                        if let Some(idx) = archetype.find_component_index_of::<$T>() {
                            *layout.component_offsets().add(idx)
                        } else {
                            return None
                        }
                    }),*
                ))}
            }

            #[inline(always)]
            fn get_refs<'a>(
                page: &'a ArchetypeDataPage,
                entity_index: usize,
                ($( paste!([<$T:lower>]) ),*): &<($($T),*) as ComponentTuple>::OffsetTuple
            ) -> Self::RefTuple<'a> {
                unsafe {(
                    $(
                        &*(
                            page.get_component_data_ptr(
                                entity_index, *paste!{ [<$T:lower>] } ,
                                std::mem::size_of::<$T>()
                            ) as *const $T
                        )
                    ),*
                )}
            }

            #[inline(always)]
            fn get_refs_mut<'a>(
                page: &'a ArchetypeDataPage,
                entity_index: usize,
                ($( paste!([<$T:lower>]) ),*): &<($($T),*) as ComponentTuple>::OffsetTuple
            ) -> Self::MutRefTuple<'a> {
                unsafe {(
                    $(
                        &mut *(
                            page.get_component_data_ptr_mut(
                                entity_index, *paste!{ [<$T:lower>] } ,
                                std::mem::size_of::<$T>()
                            ) as *mut $T
                        )
                    ),*
                )}
            }

            #[inline(always)]
            fn get_ptrs<'a>(
                page: &'a ArchetypeDataPage, 
                ($( paste!([<$T:lower>]) ),*): &<($($T),*) as ComponentTuple>::OffsetTuple
            ) -> Self::PtrTuple<'a> {
                (
                    $(
                        page.get_component_data_ptr(
                            0, 
                            *paste!{ [<$T:lower>] }, 
                            std::mem::size_of::<$T>()) as *const $T
                    ),*
                )
            }

            #[inline(always)]
            fn get_ptrs_mut<'a>(
                page: &'a ArchetypeDataPage,
                ($( paste!([<$T:lower>]) ),*): &<($($T),*) as ComponentTuple>::OffsetTuple
            ) -> Self::MutPtrTuple<'a> {
                (
                    $(
                        page.get_component_data_ptr(
                            0, 
                            *paste!{ [<$T:lower>] }, 
                            std::mem::size_of::<$T>()) as *mut $T
                    ),*
                )
            }

            #[inline(always)]
            fn add_to_ptrs<'a>(
                ($( paste!([<$T:lower>]) ),*): &<($($T),*) as ComponentTuple>::PtrTuple<'a>, 
                entity_index: usize
            ) -> Self::PtrTuple<'a> {
                    unsafe {(
                        $(
                            paste!{ [<$T:lower>] }.add(entity_index)
                        ),*
                    )}

            }

            #[inline(always)]
            fn add_to_ptrs_mut<'a>(
                ($( paste!([<$T:lower>]) ),*): &<($($T),*) as ComponentTuple>::MutPtrTuple<'a>, 
                entity_index: usize
            ) -> Self::MutPtrTuple<'a> {
                    unsafe {(
                        $(
                            paste!{ [<$T:lower>] }.add(entity_index)
                        ),*
                    )}
            }

            #[inline(always)]
            fn null_ptrs<'a>() -> Self::PtrTuple<'a> {
                ($(std::ptr::null::<$T>()),*)
            }

            #[inline(always)]
            fn null_ptrs_mut<'a>() -> Self::MutPtrTuple<'a> {
                ($(std::ptr::null_mut::<$T>()),*)
            }

            #[inline(always)]
            fn ptrs_to_refs<'a>(
                ($( paste!([<$T:lower>]) ),*): <($($T),*) as ComponentTuple>::PtrTuple<'a>,
            ) -> Self::RefTuple<'a> {
                unsafe {(
                    $(
                        &*paste!{ [<$T:lower>] }
                    ),*
                )}
            }
            
            #[inline(always)]
            fn ptrs_to_refs_mut<'a>(
                ($( paste!([<$T:lower>]) ),*): <($($T),*) as ComponentTuple>::MutPtrTuple<'a>,
            ) -> Self::MutRefTuple<'a> {
                unsafe {(
                    $(
                        &mut *paste!{ [<$T:lower>] }
                    ),*
                )}
            }

            #[inline(always)]
            fn assign_to_refs<'a>(
                ($( paste!([<$T:lower>]) ),*): <($($T),*) as ComponentTuple>::MutRefTuple<'a>,
                ($( paste!([<v_$T:lower>]) ),*): Self
            ) {
                (
                    $(
                        *paste!{ [<$T:lower>] } = paste!{ [<v_$T:lower>] }
                    ),*
                );
            }
            
            fn into_archetype() -> $crate::Archetype
            {
                let components = &mut $crate::component_types!( $($T),* );
                components.sort_by_key(|a| a.id());
                $crate::Archetype::new(components)
            }

            #[inline(always)]
            fn is_archetype_include_types(archetype: &Archetype) -> bool {
                archetype.is_include_ids(&type_ids!($($T),*))
            }

            fn add_components_to_archetype_builder(
                builder: &mut $crate::ArchetypeBuilder) -> &mut $crate::ArchetypeBuilder
            {
                let components = & $crate::component_types!( $($T),* );
                builder.include_component_types(components)
            }

            fn remove_components_from_archetype_builder(
                builder: &mut $crate::ArchetypeBuilder
            ) -> &mut $crate::ArchetypeBuilder {
                let components = & $crate::component_types!( $($T),* );
                builder.exclude_component_types(components)
            }
        }
    };

    (@type_to_usize, $T: tt) => { usize };
}

components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7, T8);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6, T7);
components_tuple_impl!(T0, T1, T2, T3, T4, T5, T6);
components_tuple_impl!(T0, T1, T2, T3, T4, T5);
components_tuple_impl!(T0, T1, T2, T3, T4);
components_tuple_impl!(T0, T1, T2, T3);
components_tuple_impl!(T0, T1, T2);
components_tuple_impl!(T0, T1);
components_tuple_impl!(T0);
