use crate::catalog::Catalog;
use crate::parser::Statement;

#[derive(Debug, Clone)]
pub enum QueryPlan {
    CreateTable {
        table: String,
        columns: Vec<(String, String)>,
    },
    DropTable {
        table: String,
    },
    Insert {
        table: String,
        columns: Vec<(String, String)>,
    },
    Select {
        table: String,
        columns: Vec<String>,
        where_clause: Option<String>,
    },
    Delete {
        table: String,
        where_clause: Option<String>,
    },
    Update {
        table: String,
        assignments: Vec<(String, String)>,
        where_clause: Option<String>,
    },
}

pub struct QueryPlanner;

impl QueryPlanner {
    pub fn plan(statement: &Statement, _catalog: &Catalog) -> QueryPlan {
        match statement {
            Statement::Create(stmt) => QueryPlan::CreateTable {
                table: stmt.table.clone(),
                columns: stmt.columns.iter().map(|c| (c.name.clone(), format!("{:?}", c.col_type))).collect(),
            },
            Statement::Drop(stmt) => QueryPlan::DropTable {
                table: stmt.table.clone(),
            },
            Statement::Insert(stmt) => QueryPlan::Insert {
                table: stmt.table.clone(),
                columns: stmt.values.iter().enumerate().map(|(i, _)| (format!("col_{}", i), "val".into())).collect(),
            },
            Statement::Select(stmt) => QueryPlan::Select {
                table: stmt.table.clone(),
                columns: stmt.columns.iter().filter_map(|c| match c {
                    crate::parser::SelectColumn::All => Some("*".into()),
                    crate::parser::SelectColumn::Named(n) => Some(n.clone()),
                }).collect(),
                where_clause: stmt.where_clause.as_ref().map(|_| "where_clause".into()),
            },
            Statement::Delete(stmt) => QueryPlan::Delete {
                table: stmt.table.clone(),
                where_clause: stmt.where_clause.as_ref().map(|_| "where_clause".into()),
            },
            Statement::Update(stmt) => QueryPlan::Update {
                table: stmt.table.clone(),
                assignments: stmt.assignments.iter().map(|(k, _)| (k.clone(), "val".into())).collect(),
                where_clause: stmt.where_clause.as_ref().map(|_| "where_clause".into()),
            },
        }
    }
}
