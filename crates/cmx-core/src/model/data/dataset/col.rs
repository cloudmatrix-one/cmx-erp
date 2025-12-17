use std::collections::HashMap;

use crate::{model::data::cell::CellValue, model::meta::fields::SYS_OBJCOLS};
use serde::{Deserialize, Serialize};
// use thiserror::Error;



#[derive(Debug, Serialize, Deserialize,Clone,Default)]
pub struct ColumnDef {
    data: HashMap<SYS_OBJCOLS, CellValue>,
}
impl ColumnDef {
    pub fn get(&self, field: &SYS_OBJCOLS) -> Option<&CellValue> {
        self.data.get(field)
    }
    pub fn set(&mut self, field: SYS_OBJCOLS, value: CellValue) {
        self.data.insert(field, value);
    }
    pub fn col_id(&self) -> String {
        self.get(&SYS_OBJCOLS::COL_ID)
            .map(|v| v.to_string())
            .unwrap_or_default()
    }
    pub fn col_isfkey(&self) -> Option<bool> {
        self.get(&SYS_OBJCOLS::COL_ISFKEY)
            .and_then(|v| v.as_bool())
    }
}
