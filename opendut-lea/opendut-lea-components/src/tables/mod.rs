pub mod selection_table;
pub mod overview_table;
pub mod multiple_selection_table;

#[derive(Default, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TableDisplayType {
    #[default]
    Text,
    Tag
}
