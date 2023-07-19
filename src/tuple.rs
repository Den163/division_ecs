use paste::paste;

use crate::archetype_data_page::ArchetypeDataPage;

pub trait ComponentsTuple<'a> {
    type OffsetsTuple;
    type RefsTuple;
}

pub(crate) trait ComponentsRefsTuple<'a, T> where T: ComponentsTuple<'a> {
    fn get_refs(page: &'a ArchetypeDataPage, entity_index: usize, offsets: &T::OffsetsTuple) -> T::RefsTuple;
}

macro_rules! components_tuple_impl {
    ($($T:ident),*) => {
        impl<'a, $($T: 'static,)*> ComponentsTuple<'a> for ($($T,)*) {
            type OffsetsTuple = ($(components_tuple_impl!(@type_to_usize, $T),)*);
            type RefsTuple = ($(&'a $T,)*);
        }

        impl<'a, $($T: 'static,)*> ComponentsRefsTuple<'a, ($($T,)*)> for ($($T,)*) {
            #[inline(always)]
            fn get_refs(
                page: &'a ArchetypeDataPage, 
                entity_index: usize, 
                ($( paste!([<$T:lower>]) ,) *): &<($($T,)*) as ComponentsTuple>::OffsetsTuple
            ) -> <($($T,)*) as ComponentsTuple<'a>>::RefsTuple {
                unsafe {(
                    $( &*(page.get_component_data_ptr(entity_index, *paste!{ [<$T:lower>] } , std::mem::size_of::<$T>()) as *const $T), )*
                )}
            }
        }
    };

    (@type_to_usize, $T: tt) => { usize };

    (@tuple_element, $T: ident) => { paste! { [<$T:lower>] } };
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