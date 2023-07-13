
pub(crate) trait TupleOfSliceToTupleOfElementRef<TResult> {
    fn as_refs_tuple(self, index: usize) -> TResult;
}