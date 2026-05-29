pub mod catalog;
pub mod executor;
pub mod parser;
pub mod planner;

use common_protocol::sql::{SqlQuery, SqlResult};
use tracing::info;

use crate::catalog::Catalog;
use crate::executor::QueryExecutor;
use crate::parser::parse_sql;
use crate::planner::QueryPlanner;

pub struct SqlEngine {
    catalog: Catalog,
}

impl SqlEngine {
    pub fn new() -> Self {
        Self {
            catalog: Catalog::new(),
        }
    }

    pub fn execute(&mut self, query: &SqlQuery) -> SqlResult {
        info!(target: "sql-engine", "Executing: {}", query.query);

        let statement = match parse_sql(&query.query) {
            Ok(stmt) => stmt,
            Err(e) => {
                return SqlResult {
                    columns: vec![],
                    rows: vec![],
                    affected_rows: 0,
                    error: Some(format!("Parse error: {}", e)),
                };
            }
        };

        let plan = QueryPlanner::plan(&statement, &self.catalog);
        let mut executor = QueryExecutor::new(&mut self.catalog);
        executor.execute(&plan)
    }
}

impl Default for SqlEngine {
    fn default() -> Self {
        Self::new()
    }
}
