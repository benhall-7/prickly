use crate::comp::Tree;
use tui_components::tui::widgets::TableState;

pub struct RouteInfo {
    pub table: TableState,
    pub index: usize,
    pub name: String,
}

impl RouteInfo {
    pub fn new(tree: &Tree) -> Self {
        let table = tree.table_state().clone();
        let row = tree.current_row().unwrap();
        RouteInfo {
            table,
            index: row.index,
            name: row.name.clone(),
        }
    }

    pub fn index(&self) -> usize {
        self.table.selected().unwrap()
    }
}
