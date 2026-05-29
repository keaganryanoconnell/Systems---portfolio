use std::collections::HashMap;

pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
}

pub struct ColumnInfo {
    pub name: String,
    pub col_type: ColumnType,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColumnType {
    Int,
    Text,
    Float,
    Bool,
}

pub struct Catalog {
    tables: HashMap<String, TableSchema>,
}

impl Catalog {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    pub fn create_table(&mut self, name: &str, columns: Vec<ColumnInfo>) {
        self.tables.insert(
            name.to_lowercase(),
            TableSchema {
                name: name.to_string(),
                columns,
            },
        );
    }

    pub fn get_table(&self, name: &str) -> Option<&TableSchema> {
        self.tables.get(&name.to_lowercase())
    }

    pub fn drop_table(&mut self, name: &str) -> bool {
        self.tables.remove(&name.to_lowercase()).is_some()
    }
}

impl Default for Catalog {
    fn default() -> Self {
        Self::new()
    }
}
