use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlQuery {
    pub query: String,
    pub params: Vec<SqlValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SqlValue {
    Null,
    Int(i64),
    Text(String),
    Float(f64),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlResult {
    pub columns: Vec<ColumnDef>,
    pub rows: Vec<Row>,
    pub affected_rows: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub values: Vec<SqlValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPlan {
    pub plan_type: String,
    pub table: String,
    pub index_used: Option<String>,
    pub estimated_rows: u64,
}
