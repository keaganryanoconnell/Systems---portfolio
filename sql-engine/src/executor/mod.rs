use common_protocol::sql::{ColumnDef, SqlResult};

use crate::catalog::{Catalog, ColumnInfo, ColumnType};
use crate::planner::QueryPlan;

pub struct QueryExecutor<'a> {
    catalog: &'a mut Catalog,
}

impl<'a> QueryExecutor<'a> {
    pub fn new(catalog: &'a mut Catalog) -> Self {
        Self { catalog }
    }

    pub fn execute(&mut self, plan: &QueryPlan) -> SqlResult {
        match plan {
            QueryPlan::CreateTable { table, columns } => {
                let cols: Vec<ColumnInfo> = columns
                    .iter()
                    .map(|(name, col_type)| ColumnInfo {
                        name: name.clone(),
                        col_type: match col_type.as_str() {
                            "Int" => ColumnType::Int,
                            "Text" => ColumnType::Text,
                            "Float" => ColumnType::Float,
                            "Bool" => ColumnType::Bool,
                            _ => ColumnType::Text,
                        },
                        nullable: true,
                    })
                    .collect();

                let schema = self.catalog.get_table(table);
                if schema.is_some() {
                    return SqlResult {
                        columns: vec![],
                        rows: vec![],
                        affected_rows: 0,
                        error: Some(format!("Table '{}' already exists", table)),
                    };
                }

                let col_defs: Vec<ColumnDef> = columns
                    .iter()
                    .map(|(name, col_type)| ColumnDef {
                        name: name.clone(),
                        col_type: col_type.clone(),
                    })
                    .collect();

                self.catalog.create_table(table, cols);

                SqlResult {
                    columns: col_defs,
                    rows: vec![],
                    affected_rows: 0,
                    error: None,
                }
            }

            QueryPlan::DropTable { table } => {
                let existed = self.catalog.drop_table(table);
                SqlResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: if existed { 1 } else { 0 },
                    error: if existed {
                        None
                    } else {
                        Some(format!("Table '{}' does not exist", table))
                    },
                }
            }

            QueryPlan::Insert { table, .. } => {
                let schema = match self.catalog.get_table(table) {
                    Some(s) => s,
                    None => {
                        return SqlResult {
                            columns: vec![],
                            rows: vec![],
                            affected_rows: 0,
                            error: Some(format!("Table '{}' does not exist", table)),
                        }
                    }
                };

                let col_defs: Vec<ColumnDef> = schema
                    .columns
                    .iter()
                    .map(|c| ColumnDef {
                        name: c.name.clone(),
                        col_type: format!("{:?}", c.col_type),
                    })
                    .collect();

                SqlResult {
                    columns: col_defs,
                    rows: vec![],
                    affected_rows: 1,
                    error: None,
                }
            }

            QueryPlan::Select { table, columns, .. } => {
                let schema = match self.catalog.get_table(table) {
                    Some(s) => s,
                    None => {
                        return SqlResult {
                            columns: vec![],
                            rows: vec![],
                            affected_rows: 0,
                            error: Some(format!("Table '{}' does not exist", table)),
                        }
                    }
                };

                let col_defs: Vec<ColumnDef> = if columns.contains(&"*".to_string()) {
                    schema
                        .columns
                        .iter()
                        .map(|c| ColumnDef {
                            name: c.name.clone(),
                            col_type: format!("{:?}", c.col_type),
                        })
                        .collect()
                } else {
                    columns
                        .iter()
                        .map(|n| ColumnDef {
                            name: n.clone(),
                            col_type: "Text".into(),
                        })
                        .collect()
                };

                SqlResult {
                    columns: col_defs,
                    rows: vec![],
                    affected_rows: 0,
                    error: None,
                }
            }

            QueryPlan::Delete { table, .. } => {
                if self.catalog.get_table(table).is_none() {
                    return SqlResult {
                        columns: vec![],
                        rows: vec![],
                        affected_rows: 0,
                        error: Some(format!("Table '{}' does not exist", table)),
                    };
                }

                SqlResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                    error: None,
                }
            }

            QueryPlan::Update { table, .. } => {
                if self.catalog.get_table(table).is_none() {
                    return SqlResult {
                        columns: vec![],
                        rows: vec![],
                        affected_rows: 0,
                        error: Some(format!("Table '{}' does not exist", table)),
                    };
                }

                SqlResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                    error: None,
                }
            }
        }
    }
}
