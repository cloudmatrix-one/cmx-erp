use std::sync::Arc;
use tokio::sync::RwLock;

pub mod cell;
pub mod col;
pub mod row;
pub mod head;

use head::Head;
use row::Row;
use col::Col;

pub struct Area {
    pub head: Arc<RwLock<Head>>,
    pub rows: Arc<RwLock<Vec<Arc<Row>>>>,
    pub cols: Arc<RwLock<Vec<Arc<Col>>>>,
}

impl Area {
    pub fn new(head: Head) -> Self {
        Area {
            head: Arc::new(RwLock::new(head)),
            rows: Arc::new(RwLock::new(Vec::new())),
            cols: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn add_row(&self, row: Row) {
        let mut rows = self.rows.write().await;
        rows.push(Arc::new(row));
    }

    pub async fn add_col(&self, col: Col) {
        let mut cols = self.cols.write().await;
        cols.push(Arc::new(col));
    }
}
