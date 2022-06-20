use cw_storage_plus::MultiIndex;

pub trait OrderIndices<'a, T> {
    fn owner_index(&self) -> &MultiIndex<'a, String, T, String>;
    fn type_index(&self) -> &MultiIndex<'a, String, T, String>;
}
