pub mod rds;
// pub mod db;
// pub mod seaorm;
pub mod col;
pub mod row;
// pub mod cds;
// pub mod idme_metamodel;



// use chrono::NaiveDateTime;
use col::ColumnDef;
use row::RowSet;
// use rust_decimal::Decimal;
use thiserror::Error;
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
// use chrono::NaiveDateTime;
// use rust_decimal::Decimal;
use std::collections::HashMap;
// use strum_macros::{EnumString, Display};

use crate::model::meta::fields::SYS_OBJECTS;

use super::cell::CellValue;



#[derive(Debug, Serialize, Deserialize,Clone)]
pub enum ColumnType {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Decimal,
    String,
}
impl Default for ColumnType {
    fn default() -> Self {
        ColumnType::String
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TableSchema {
    data: HashMap<SYS_OBJECTS, CellValue>,
    pub columns: Vec<ColumnDef>,
    #[serde(skip)]
    column_indices: Vec<String>,
}

impl TableSchema {
    pub fn get_column_index(&self, column_name: &str) -> Option<usize> {
        self.column_indices.iter().position(|name| name == column_name)
    }

    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    // 添加便捷访问方法
    pub fn get(&self, field: &SYS_OBJECTS) -> Option<&CellValue> {
        self.data.get(field)
    }

    pub fn set(&mut self, field: SYS_OBJECTS, value: CellValue) {
        self.data.insert(field, value);
    }

    // 常用字段的专用访问器
    pub fn obj_id(&self) -> Option<&str> {
        self.get(&SYS_OBJECTS::OBJ_ID).and_then(|v| v.as_str())
    }

    /// 检查对象是否启用
    /// 如果 F_ENABLE 字段值为字符串 "1" 则返回 true，否则返回 false
    pub fn is_enabled(&self) -> bool {
        self.get(&SYS_OBJECTS::F_ENABLE)
        .and_then(|v| v.as_str())
        .map_or(false, |s| s == "1") 
    }
    
    /// 检查是否支持多语言
    pub fn is_multilingual(&self) -> bool {
        self.get(&SYS_OBJECTS::OBJ_LANG)
        .and_then(|v| v.as_str())
        .map_or(false, |s| s == "1") 
    }
    
    /// 检查是否支持多单位
    pub fn is_multi_unit(&self) -> bool {
        self.get(&SYS_OBJECTS::OBJ_MUNIT)
            .and_then(|v| v.as_str())
            .map_or(false, |s| s == "1") 
    }
    
    /// 检查是否有自定义列
    pub fn has_custom_columns(&self) -> bool {
        self.get(&SYS_OBJECTS::OBJ_CSTCOL)
        .and_then(|v| v.as_str())
        .map_or(false, |s| s == "1") 
    }
    
    /// 检查是否为单位对象
    pub fn is_unit_object(&self) -> bool {
        self.get(&SYS_OBJECTS::OBJ_UNIT)
        .and_then(|v| v.as_str())
        .map_or(false, |s| s == "1") 
    }
}

// Builder 模式
#[derive(Default)]
pub struct TableSchemaBuilder {
    data: HashMap<SYS_OBJECTS, CellValue>,
    columns: Vec<ColumnDef>,
}

impl TableSchemaBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_obj_id(mut self, value: String) -> Self {
        self.data.insert(SYS_OBJECTS::OBJ_ID, CellValue::String(value));
        self
    }

    pub fn with_obj_mc(mut self, value: String) -> Self {
        self.data.insert(SYS_OBJECTS::OBJ_MC, CellValue::String(value));
        self
    }

    // ... 其他字段的 builder 方法

    pub fn with_columns(mut self, columns: Vec<ColumnDef>) -> Self {
        self.columns = columns;
        self
    }

    pub fn build(self) -> TableSchema {
        let column_indices: Vec<String> = self.columns.iter()
            .map(|col| col.col_id())
            .collect();

        TableSchema {
            data: self.data,
            columns: self.columns,
            column_indices,
        }
    }
}


#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct DataSet {
    rows: Option<Vec<RowSet>>,
}

impl DataSet {
    pub fn new() -> Self {
        Self {
            rows: Some(Vec::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Some(Vec::with_capacity(capacity)),
        }
    }

    // 添加一行数据
    pub fn add_row(&mut self, schema: &TableSchema, values: Vec<CellValue>) -> Result<(), DataSetError> {
        if values.len() != schema.column_count() {
            return Err(DataSetError::ColumnCountMismatch);
        }

        match &mut self.rows {
            Some(rows) => {
                let mut row = RowSet::new(schema.column_count());
                for (i, value) in values.into_iter().enumerate() {
                    row.set_value(i, value).unwrap();
                }
                rows.push(row);
                Ok(())
            }
            None => Err(DataSetError::RowsNotInitialized)
        }
    }

    // 获取指定行
    pub fn get_row(&self, index: usize) -> Result<&RowSet, DataSetError> {
        self.rows
            .as_ref()
            .ok_or(DataSetError::RowsNotInitialized)?
            .get(index)
            .ok_or(DataSetError::IndexOutOfBounds)
    }

    // 获取指定行的变引用
    pub fn get_row_mut(&mut self, index: usize) -> Result<&mut RowSet, DataSetError> {
        self.rows
            .as_mut()
            .ok_or(DataSetError::RowsNotInitialized)?
            .get_mut(index)
            .ok_or(DataSetError::IndexOutOfBounds)
    }

    // // 获取指定单元格的值
    // pub fn get_cell(&self, schema: &TableSchema, row_index: usize, column_name: &str) -> Result<&CellValue, DataSetError> {
    //     let col_index = schema.get_column_index(column_name)
    //         .ok_or(DataSetError::ColumnNotFound)?;
        
    //     let row = self.get_row(row_index)?;
    //     row.get_value(col_index)
    // }

    // // 设置指定单元格的值
    // pub fn set_cell(&mut self, schema: &TableSchema, row_index: usize, column_name: &str, value: CellValue) -> Result<(), DataSetError> {
    //     let col_index = schema.get_column_index(column_name)
    //         .ok_or(DataSetError::ColumnNotFound)?;
        
    //     let row = self.get_row_mut(row_index)?;
    //     row.set_value(col_index, value)
    // }

    // 删除指定行
    pub fn remove_row(&mut self, index: usize) -> Result<RowSet, DataSetError> {
        match &mut self.rows {
            Some(rows) if index < rows.len() => {
                Ok(rows.remove(index))
            }
            Some(_) => Err(DataSetError::IndexOutOfBounds),
            None => Err(DataSetError::RowsNotInitialized)
        }
    }

    // 获取行数
    pub fn row_count(&self) -> usize {
        self.rows.as_ref().map_or(0, |rows| rows.len())
    }

    // 清空所有行
    pub fn clear(&mut self) -> Result<(), DataSetError> {
        match &mut self.rows {
            Some(rows) => {
                rows.clear();
                Ok(())
            }
            None => Err(DataSetError::RowsNotInitialized)
        }
    }

    // 批量添加行
    pub fn add_rows(&mut self, schema: &TableSchema, rows_data: Vec<Vec<CellValue>>) -> Result<(), DataSetError> {
        for row_data in rows_data {
            self.add_row(schema, row_data)?;
        }
        Ok(())
    }

    // // 获取指定列的所有值
    // pub fn get_column_values(&self, schema: &TableSchema, column_name: &str) -> Result<Vec<&CellValue>, DataSetError> {
    //     let col_index = schema.get_column_index(column_name)
    //         .ok_or(DataSetError::ColumnNotFound)?;

    //     match &self.rows {
    //         Some(rows) => {
    //             let values: Vec<&CellValue> = rows.iter()
    //                 .map(|row| row.get_value(col_index))
    //                 .collect::<Result<Vec<&CellValue>, DataSetError>>()
    //                 .unwrap();
    //             Ok(values)
    //         }
    //         None => Err(DataSetError::RowsNotInitialized)
    //     }
    // }

    // 根据指定列名数组对行进行排序，返回排序后的行引用数组
    pub fn sort_rows<'a>(&'a self, schema: &TableSchema, sort_columns: &[&str]) -> Result<Vec<&'a RowSet>, DataSetError> {
        let rows = self.rows.as_ref().ok_or(DataSetError::RowsNotInitialized)?;
        
        // 获取所有列的索引
        let column_indices: Result<Vec<usize>, DataSetError> = sort_columns
            .iter()
            .map(|&col_name| {
                schema.get_column_index(col_name)
                    .ok_or(DataSetError::ColumnNotFound)
            })
            .collect();
        let column_indices = column_indices?;

        // 创建行引用的可变数组
        let mut sorted_rows: Vec<&RowSet> = rows.iter().collect();

        // 根据指定列进行排序
        sorted_rows.sort_by(|a, b| {
            for &col_idx in &column_indices {
                // 获取两行对应列的值
                let value_a = a.get_value(col_idx);
                let value_b = b.get_value(col_idx);

                // 比较两个值
                match (value_a, value_b) {
                    (Ok(v1), Ok(v2)) => {
                        match compare_cell_values(v1, v2) {
                            Ordering::Equal => continue, // 相等时继续比较下一列
                            other => return other,      // 不相等时返回比较结果
                        }
                    }
                    // 处理错误情况
                    (Err(_), Ok(_)) => return Ordering::Greater,
                    (Ok(_), Err(_)) => return Ordering::Less,
                    (Err(_), Err(_)) => return Ordering::Equal,
                }
            }
            Ordering::Equal // 所有列都相等
        });

        Ok(sorted_rows)
    }

    // 根据指定列名数组对行进行排序，支持指定升序/降序
    pub fn sort_rows_with_order<'a>(
        &'a self, 
        schema: &TableSchema, 
        sort_specs: &[(&str, bool)]  // (列名, 是否升序)
    ) -> Result<Vec<&'a RowSet>, DataSetError> {
        let rows = self.rows.as_ref().ok_or(DataSetError::RowsNotInitialized)?;
        
        // 获取所有列的索引
        let column_specs: Result<Vec<(usize, bool)>, DataSetError> = sort_specs
            .iter()
            .map(|&(col_name, ascending)| {
                Ok((schema.get_column_index(col_name)
                    .ok_or(DataSetError::ColumnNotFound)?, ascending))
            })
            .collect();
        let column_specs = column_specs?;

        // 创建行引用的可变数组
        let mut sorted_rows: Vec<&RowSet> = rows.iter().collect();

        // 根据指定列进行排序
        sorted_rows.sort_by(|a, b| {
            for &(col_idx, ascending) in &column_specs {
                // 获取两行对应列的值
                let value_a = a.get_value(col_idx);
                let value_b = b.get_value(col_idx);

                // 比较两个值
                match (value_a, value_b) {
                    (Ok(v1), Ok(v2)) => {
                        let order = compare_cell_values(v1, v2);
                        match order {
                            Ordering::Equal => continue, // 相等时继续比较下一列
                            other => {
                                return if ascending {
                                    other
                                } else {
                                    other.reverse()
                                }
                            }
                        }
                    }
                    // 处理错误情况
                    (Err(_), Ok(_)) => return Ordering::Greater,
                    (Ok(_), Err(_)) => return Ordering::Less,
                    (Err(_), Err(_)) => return Ordering::Equal,
                }
            }
            Ordering::Equal // 所有列都相等
        });

        Ok(sorted_rows)
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    // // 行操作
    // pub fn add_row(&mut self, row: RowSet) -> Result<(), DataSetError> {
    //     self.rows.get_or_insert_with(Vec::new).push(row);
    //     Ok(())
    // }

    // pub fn get_row(&self, index: usize) -> Result<&RowSet, DataSetError> {
    //     self.rows
    //         .as_ref()
    //         .and_then(|rows| rows.get(index))
    //         .ok_or(DataSetError::IndexOutOfBounds)
    // }

    // pub fn get_row_mut(&mut self, index: usize) -> Result<&mut RowSet, DataSetError> {
    //     self.rows
    //         .as_mut()
    //         .and_then(|rows| rows.get_mut(index))
    //         .ok_or(DataSetError::IndexOutOfBounds)
    // }

    // 层级数据操作
    pub fn add_child_dataset(&mut self, row_index: usize, child_id: String, child_dataset: DataSet) -> Result<(), DataSetError> {
        let row = self.get_row_mut(row_index)?;
        row.add_child(child_id, child_dataset);
        Ok(())
    }

    pub fn get_child_dataset(&self, row_index: usize, child_id: &str) -> Result<Option<&DataSet>, DataSetError> {
        let row = self.get_row(row_index)?;
        Ok(row.get_child(child_id))
    }

    // ���历方法
    pub fn traverse<F>(&self, mut f: F) -> Result<(), DataSetError>
    where
        F: FnMut(&RowSet, usize)
    {
        self.traverse_internal(&mut f, 0)
    }

    fn traverse_internal<F>(&self, f: &mut F, depth: usize) -> Result<(), DataSetError>
    where
        F: FnMut(&RowSet, usize)
    {
        if let Some(rows) = &self.rows {
            for row in rows {
                f(row, depth);
                if let Some(children) = &row.children {
                    for (_, child_dataset) in children {
                        child_dataset.traverse_internal(f, depth + 1)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// 比较两个 CellValue 的辅助函数
///
/// 提供 serde_json::Value 的完整比较实现，按以下规则排序：
/// - Null < Bool < Number < String < Array < Object
/// - 同类型值按其自然顺序比较
/// - 数组按元素逐个比较，不同长度时短数组较小
/// - 对象按键值对数量比较
fn compare_cell_values(a: &CellValue, b: &CellValue) -> Ordering {
    use serde_json::Value;

    // 首先按类型排序
    let type_rank = |v: &Value| match v {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(_) => 2,
        Value::String(_) => 3,
        Value::Array(_) => 4,
        Value::Object(_) => 5,
    };

    let rank_a = type_rank(a);
    let rank_b = type_rank(b);

    match rank_a.cmp(&rank_b) {
        Ordering::Equal => {
            // 同类型，比较值
            match a {
                Value::Bool(x) => x.cmp(&b.as_bool().unwrap()),
                Value::Number(x) => compare_numbers(x, b.as_number().unwrap()),
                Value::String(x) => x.as_str().cmp(b.as_str().unwrap()),
                Value::Array(x) => compare_arrays(x, b.as_array().unwrap()),
                Value::Object(x) => x.len().cmp(&b.as_object().unwrap().len()),
                Value::Null => Ordering::Equal,
            }
        }
        ordering => ordering,
    }
}

/// 比较两个 JSON 数字
fn compare_numbers(a: &serde_json::Number, b: &serde_json::Number) -> Ordering {
    // 优先尝试整数比较
    if let (Some(a_int), Some(b_int)) = (a.as_i64(), b.as_i64()) {
        return a_int.cmp(&b_int);
    }

    if let (Some(a_uint), Some(b_uint)) = (a.as_u64(), b.as_u64()) {
        return a_uint.cmp(&b_uint);
    }

    // 回退到浮点数比较
    let a_float = a.as_f64().unwrap_or(0.0);
    let b_float = b.as_f64().unwrap_or(0.0);
    a_float.partial_cmp(&b_float).unwrap_or(Ordering::Equal)
}

/// 比较两个 JSON 数组
fn compare_arrays(a: &[CellValue], b: &[CellValue]) -> Ordering {
    // 先比较长度
    match a.len().cmp(&b.len()) {
        Ordering::Equal => {
            // 长度相等，逐元素比较
            for (elem_a, elem_b) in a.iter().zip(b.iter()) {
                match compare_cell_values(elem_a, elem_b) {
                    Ordering::Equal => continue,
                    ordering => return ordering,
                }
            }
            Ordering::Equal
        }
        ordering => ordering,
    }
}

#[derive(Error, Debug)]
pub enum DataSetError {
    #[error("Rows not initialized")]
    RowsNotInitialized,
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Column not found")]
    ColumnNotFound,
    #[error("Column count mismatch")]
    ColumnCountMismatch,
}

// // 添加测试
// #[cfg(test)]
// mod tests {
//     // use std::sync::Arc;

//     use col::ColumnType;

//     use super::*;

//     #[test]
//     fn test_schema_serialization() {
//         // 创建测试数据
//         let columns = vec![
//             ColumnDef {
//                 name: "id".to_string(),
//                 column_type: ColumnType::Integer,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "name".to_string(),
//                 column_type: ColumnType::Text,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "created_at".to_string(),
//                 column_type: ColumnType::Timestamp,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//         ];

//         let schema = TableSchema::new("users".to_string(), columns);

//         // 测试序列化
//         let json = schema.to_json().unwrap();
//         println!("Serialized JSON:\n{}", json);

//         // 测试反序列化
//         let deserialized_schema = TableSchema::from_json(&json).unwrap();

//         // 验证结果
//         assert_eq!(schema.table_name, deserialized_schema.table_name);
//         assert_eq!(schema.column_count(), deserialized_schema.column_count());
        
//         // 验证列索引是否正确重建
//         assert_eq!(
//             schema.get_column_index("id"),
//             deserialized_schema.get_column_index("id")
//         );
//     }

//     #[test]
//     fn test_schema_with_all_column_types() {
//         let columns = vec![
//             ColumnDef {
//                 name: "int_col".to_string(),
//                 column_type: ColumnType::Integer,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "bigint_col".to_string(),
//                 column_type: ColumnType::BigInt,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "float_col".to_string(),
//                 column_type: ColumnType::Float,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "double_col".to_string(),
//                 column_type: ColumnType::Double,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "text_col".to_string(),
//                 column_type: ColumnType::Text,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "bool_col".to_string(),
//                 column_type: ColumnType::Boolean,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "date_col".to_string(),
//                 column_type: ColumnType::Date,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "time_col".to_string(),
//                 column_type: ColumnType::Time,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//             ColumnDef {
//                 name: "timestamp_col".to_string(),
//                 column_type: ColumnType::Timestamp,
//                 nullable: false,
//                 is_foreign_key: false,
//                 foreign_table_name: None,
//                 foreign_column_name: None,
//                 multilingual: false,
//                 default_value: None,
//                 comment: None,
//             },
//         ];

//         let schema = TableSchema::new("all_types".to_string(), columns);
//         let json = schema.to_json().unwrap();
//         println!("All Types JSON:\n{}", json);

//         let deserialized = TableSchema::from_json(&json).unwrap();
//         assert_eq!(schema.column_count(), deserialized.column_count());
//     }

//     #[test]
//     fn test_dataset_serialization() {
//         // Create test schema
//         let columns = vec![
//             ColumnDef::new("id".into(), ColumnType::Integer),
//             ColumnDef::new("name".into(), ColumnType::Text),
//         ];
//         let schema = TableSchema::new("test".to_string(), columns);

//         // Create and populate dataset
//         let mut dataset = DataSet::new();
//         dataset.add_row(&schema, vec![
//             CellValue::Integer(1),
//             CellValue::Text(Option::Some("Alice".to_string())),
//         ]).unwrap();
//         dataset.add_row(&schema, vec![
//             CellValue::Integer(2),
//             CellValue::Text(Option::Some("Bob".to_string())),
//         ]).unwrap();

//         // Test serialization
//         let json = dataset.to_json().unwrap();
//         println!("Serialized DataSet:\n{}", json);

//         // Test deserialization
//         let deserialized_dataset = DataSet::from_json(&json).unwrap();

//         // Verify results
//         assert_eq!(dataset.row_count(), deserialized_dataset.row_count());
        
//         // Compare each row's values
//         // for i in 0..dataset.row_count() {
//         //     let original_row = dataset.get_row(i).unwrap();
//         //     let deserialized_row = deserialized_dataset.get_row(i).unwrap();
//         //     for j in 0..schema.column_count() {
//         //         // let original_value = original_row.get_value(j);
//         //         // let deserialized_value = deserialized_row.get_value(j);
//         //         // assert_eq!(original_value, deserialized_value);
//         //     }
//         // }
//     }
// }
