use std::collections::HashMap;
use serde::{Serialize, Deserialize};
// use super::{cell::CellValue, DataSet};
use thiserror::Error;

use crate::model::data::cell::CellValue;

use super::DataSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowSet {
    // 行数据（顺序存储很重要，不能打乱，依赖于列定义的顺序）
    values: Vec<CellValue>,
    // 子数据集（Hash的key是数据集的ID）
    #[serde(default)]  // 序列化时如果为None则使用默认值
    pub children: Option<HashMap<String, DataSet>>,
}

impl RowSet {
    pub fn new(column_count: usize) -> Self {
        Self {
            values: Vec::with_capacity(column_count),
            children: None,
        }
    }

    // 子数据集操作
    pub fn add_child(&mut self, id: String, dataset: DataSet) {
        if self.children.is_none() {
            self.children = Some(HashMap::new());
        }
        if let Some(children) = &mut self.children {
            children.insert(id, dataset);
        }
    }

    pub fn get_child(&self, id: &str) -> Option<&DataSet> {
        self.children.as_ref()
            .and_then(|children| children.get(id))
    }

    pub fn get_child_mut(&mut self, id: &str) -> Option<&mut DataSet> {
        self.children.as_mut()
            .and_then(|children| children.get_mut(id))
    }

    pub fn remove_child(&mut self, id: &str) -> Option<DataSet> {
        self.children.as_mut()
            .and_then(|children| children.remove(id))
    }

    pub fn child_ids(&self) -> Vec<String> {
        self.children.as_ref()
            .map(|children| children.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn has_children(&self) -> bool {
        self.children.as_ref()
            .map(|children| !children.is_empty())
            .unwrap_or(false)
    }

    // 遍历子数据集
    pub fn traverse_children<F>(&self, f: &mut F) -> Result<(), RowSetError>
    where
        F: FnMut(&str, &DataSet)
    {
        if let Some(children) = &self.children {
            for (id, dataset) in children {
                f(id, dataset);
            }
        }
        Ok(())
    }

    // 现有的值操作方法
    pub fn set_value(&mut self, index: usize, value: CellValue) -> Result<(), RowSetError> {
        if index >= self.values.len() && index == self.values.len() {
            self.values.push(value);
            Ok(())
        } else if index < self.values.len() {
            self.values[index] = value;
            Ok(())
        } else {
            Err(RowSetError::IndexOutOfBounds)
        }
    }

    pub fn get_value(&self, index: usize) -> Result<&CellValue, RowSetError> {
        self.values.get(index)
            .ok_or(RowSetError::IndexOutOfBounds)
    }

    pub fn get_value_mut(&mut self, index: usize) -> Result<&mut CellValue, RowSetError> {
        self.values.get_mut(index)
            .ok_or(RowSetError::IndexOutOfBounds)
    }

    // 序列化相关方法
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    // 克隆行但不包含子数据集
    pub fn clone_without_children(&self) -> Self {
        Self {
            values: self.values.clone(),
            children: None,
        }
    }

    // 合并子数据集
    pub fn merge_children(&mut self, other: &Self) -> Result<(), RowSetError> {
        if let Some(other_children) = &other.children {
            for (id, dataset) in other_children {
                self.add_child(id.clone(), dataset.clone());
            }
        }
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum RowSetError {
    #[error("Index out of bounds")]
    IndexOutOfBounds,
    #[error("Child dataset not found: {0}")]
    ChildNotFound(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

// // 测试代码
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::data::dataset::{col::{ColumnDef, ColumnType}, TableSchema};

//     #[test]
//     fn test_rowset_with_children() {
//         // 创建主行
//         let mut row = RowSet::new(2);
//         row.set_value(0, CellValue::Integer(1)).unwrap();
//         row.set_value(1, CellValue::Text(Some("Parent".to_string()))).unwrap();

//         // 创建子数据集
//         let mut child_dataset = DataSet::new();
//         let mut child_row = RowSet::new(2);
//         child_row.set_value(0, CellValue::Integer(101)).unwrap();
//         child_row.set_value(1, CellValue::Text(Some("Child".to_string()))).unwrap();
//         child_dataset.add_row(&TableSchema::new(
//             "child".to_string(),
//             vec![
//                 ColumnDef::new("id".into(), ColumnType::Integer),
//                 ColumnDef::new("name".into(), ColumnType::Text),
//             ]
//         ), vec![
//             CellValue::Integer(101),
//             CellValue::Text(Some("Child".to_string())),
//         ]).unwrap();

//         // 添加子数据集
//         row.add_child("child1".to_string(), child_dataset);

//         // 验证子数据集
//         assert!(row.has_children());
//         assert_eq!(row.child_ids().len(), 1);
//         assert!(row.get_child("child1").is_some());

//         // 测试序列化
//         let json = row.to_json().unwrap();
//         let deserialized = RowSet::from_json(&json).unwrap();
//         assert!(deserialized.has_children());
//         assert_eq!(deserialized.child_ids().len(), 1);

//         // 测试遍历
//         row.traverse_children(&mut |id, dataset| {
//             assert_eq!(id, "child1");
//             assert_eq!(dataset.row_count(), 1);
//         }).unwrap();
//     }

//     #[test]
//     fn test_rowset_operations() {
//         let mut row = RowSet::new(2);
        
//         // 测试基本值操作
//         row.set_value(0, CellValue::Integer(1)).unwrap();
//         assert_eq!(
//             matches!(row.get_value(0).unwrap(), CellValue::Integer(1)),
//             true
//         );

//         // 测试子数据集操作
//         let child_dataset = DataSet::new();
//         row.add_child("child1".to_string(), child_dataset.clone());
//         row.add_child("child2".to_string(), child_dataset.clone());

//         assert_eq!(row.child_ids().len(), 2);
//         assert!(row.get_child("child1").is_some());
        
//         // 测试移除子数据集
//         let removed = row.remove_child("child1");
//         assert!(removed.is_some());
//         assert_eq!(row.child_ids().len(), 1);

//         // 测试克隆不包含子数据集
//         let cloned = row.clone_without_children();
//         assert!(!cloned.has_children());
//         assert_eq!(
//             matches!(cloned.get_value(0).unwrap(), CellValue::Integer(1)),
//             true
//         );
//     }
// }