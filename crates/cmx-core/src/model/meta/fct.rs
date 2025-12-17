use std::{collections::HashMap, sync::Arc, sync::RwLock};
use lazy_static::lazy_static;

use crate::model::data::{dataset::TableSchema, KeyValue};

use super::dct::DCTMeta;

use serde::{Deserialize, Serialize};

lazy_static! {
    static ref FCT_METAS: RwLock<HashMap<String, Arc<FCTMeta>>> = RwLock::new(HashMap::new());
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FCTMeta {
    pub id: String,                // ID
    pub name: String,              // Name
    pub info: Option<KeyValue>, 
    pub table_schema: TableSchema,  // Changed from Arc<TableSchema>
    #[serde(skip)]
    pub dct_metas: Option<HashMap<String, Arc<DCTMeta>>>,     // 所有用到的DCTMeta，String为FCT的某一列，此列引用了DCTMeta
    pub settings: Option<KeyValue>, // 可选的 KeyValue 成员
}

#[derive(Debug)]
pub struct FCTMetaManager;

impl FCTMetaManager {

    pub fn add_meta(meta: Arc<FCTMeta>) {
        let mut metas = FCT_METAS.write().unwrap();
        metas.insert(meta.id.clone(), meta);
    }

    pub fn get_meta(id: &str) -> Option<Arc<FCTMeta>> {
        let metas = FCT_METAS.read().unwrap();
        metas.get(id).cloned()
    }

    // Optional: Add a clear method for testing
    #[cfg(test)]
    pub fn clear() {
        let mut metas = FCT_METAS.write().unwrap();
        metas.clear();
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::data::dataset::{TableSchema, col::{ColumnDef, ColumnType}};
//     use crate::data::cell::CellValue;

//     fn create_test_schema() -> TableSchema {
//         let columns = vec![
//             ColumnDef::new("id".to_string(), ColumnType::Integer)
//                 .with_nullable(false),
//             ColumnDef::new("name".to_string(), ColumnType::Text)
//                 .with_nullable(false),
//             ColumnDef::new("value".to_string(), ColumnType::Double)
//                 .with_nullable(true),
//         ];
//         TableSchema::new("test_table".to_string(), columns)
//     }

//     #[test]
//     fn test_fct_meta_basic_serialization() {
//         let schema = create_test_schema();
        
//         // 创建一个基本的 FCTMeta
//         let fct_meta = FCTMeta {        
//             id: "fct1".to_string(),
//             name: "Test FCT".to_string(),
//             info: None,
//             table_schema: schema,
//             dct_metas: None,
//             settings: None,
//         };

//         // 序列化为 JSON
//         let json = serde_json::to_string(&fct_meta).expect("Failed to serialize");
        
//         // 从 JSON 反序列化
//         let deserialized: FCTMeta = serde_json::from_str(&json).expect("Failed to deserialize");
        
//         // 验证基本字段
//         assert_eq!(deserialized.id, "fct1");
//         assert_eq!(deserialized.name, "Test FCT");
//         assert_eq!(deserialized.table_schema.column_count(), 3);
//         assert!(deserialized.dct_metas.is_none());
//         assert!(deserialized.settings.is_none());
//     }

//     #[test]
//     fn test_fct_meta_with_dct_and_keyvalue() {
//         let schema = create_test_schema();
        
//         // 创建 DCTMeta
//         let mut dct_metas = HashMap::new();
//         dct_metas.insert(
//             "dim1".to_string(),
//             Arc::new(DCTMeta {
//                 id: "dct1".to_string(),
//                 name: "Dimension 1".to_string(),
//                 // encoding_structure: "default".to_string(),
//                 // data_permission_id: "perm1".to_string(),
//                 // hierarchy_type: "standard".to_string(),
//                 table_schema: schema.clone(),
//                 settings: None,
//                 dct_metas: None,
//                 info: None,
//             })
//         );

//         // 创建 KeyValue
//         let mut key_value = KeyValue::new();
//         key_value.put("type".to_string(), CellValue::Text(Option::Some("fact".to_string())));
//         key_value.put("version".to_string(), CellValue::Integer(1));

//         // 创建完整的 FCTMeta
//         let fct_meta = FCTMeta {    
//             id: "fct1".to_string(),
//             name: "Test FCT".to_string(),
//             info: None,
//             table_schema: schema,
//             dct_metas: Some(dct_metas),
//             settings: Some(key_value),
//         };

//         // 序列化为 JSON
//         let json = serde_json::to_string(&fct_meta).expect("Failed to serialize");
        
//         // 从 JSON 反序列化
//         let deserialized: FCTMeta = serde_json::from_str(&json).expect("Failed to deserialize");
        
//         // 验证基本字段
//         assert_eq!(deserialized.id, "fct1");
//         assert_eq!(deserialized.name, "Test FCT");
        
//         // 验证 DCTMeta
//         let dct_metas = deserialized.dct_metas.as_ref().expect("DCTMetas should exist");
//         let dct = dct_metas.get("dim1").expect("DCTMeta 'dim1' should exist");
//         assert_eq!(dct.id, "dct1");
//         assert_eq!(dct.name, "Dimension 1");
        
//         // 验证 KeyValue
//         let kv = deserialized.settings.as_ref().expect("KeyValue should exist");
//         match kv.get("type").expect("'type' key should exist") {
//             CellValue::Text(text) => assert_eq!(text.as_ref().unwrap().as_str(), "fact"),
//             _ => panic!("Unexpected value type for 'type'"),
//         }
//         match kv.get("version").expect("'version' key should exist") {
//             CellValue::Integer(num) => assert_eq!(*num, 1),
//             _ => panic!("Unexpected value type for 'version'"),
//         }
//     }
// }
