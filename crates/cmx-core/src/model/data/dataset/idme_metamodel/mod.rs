
pub mod core;
pub mod business_process_extensions;
pub mod industrial_extensions;
pub mod data_graph;

// 重新导出核心类型
pub use core::*;
pub use business_process_extensions::*;
pub use industrial_extensions::*;
pub use data_graph::*;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 数据类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    DateTime,
    Decimal,
    Text,
    Json,
    Reference(String), // 引用其他实体的ID
}

/// 属性定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttributeDefinition {
    pub id: String,
    pub name: String,
    pub data_type: DataType,
    pub is_required: bool,
    pub default_value: Option<serde_json::Value>,
    pub description: Option<String>,
    pub constraints: Option<HashMap<String, serde_json::Value>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AttributeDefinition {
    pub fn new(name: String, data_type: DataType, is_required: bool) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            data_type,
            is_required,
            default_value: None,
            description: None,
            constraints: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn with_default_value(mut self, value: serde_json::Value) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn with_constraint(mut self, key: String, value: serde_json::Value) -> Self {
        self.constraints.get_or_insert_with(HashMap::new).insert(key, value);
        self
    }
}

/// 6类元关系类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetaRelationType {
    /// 继承关系 - 表示类型的继承层次
    Inheritance,
    /// 组合关系 - 表示整体与部分的强关联（生命周期一致）
    Composition,
    /// 聚合关系 - 表示整体与部分的弱关联
    Aggregation,
    /// 关联关系 - 表示实体间的一般性关联
    Association,
    /// 依赖关系 - 表示实体间的依赖关系
    Dependency,
    /// 实现关系 - 表示接口与实现的关系
    Realization,
}

/// 基数约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cardinality {
    pub source_min: u32,
    pub source_max: Option<u32>, // None 表示无限制
    pub target_min: u32,
    pub target_max: Option<u32>, // None 表示无限制
}

impl Cardinality {
    pub fn one_to_one() -> Self {
        Self {
            source_min: 1,
            source_max: Some(1),
            target_min: 1,
            target_max: Some(1),
        }
    }

    pub fn one_to_many() -> Self {
        Self {
            source_min: 1,
            source_max: Some(1),
            target_min: 0,
            target_max: None,
        }
    }

    pub fn many_to_one() -> Self {
        Self {
            source_min: 0,
            source_max: None,
            target_min: 1,
            target_max: Some(1),
        }
    }

    pub fn many_to_many() -> Self {
        Self {
            source_min: 0,
            source_max: None,
            target_min: 0,
            target_max: None,
        }
    }

    pub fn zero_to_one() -> Self {
        Self {
            source_min: 0,
            source_max: Some(1),
            target_min: 0,
            target_max: Some(1),
        }
    }
}

/// 实体元模型 - 描述业务实体的结构和属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetaModel {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub description: Option<String>,
    pub attributes: HashMap<String, AttributeDefinition>,
    pub parent_entity: Option<String>, // 继承的父实体ID
    pub is_abstract: bool,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl EntityMetaModel {
    pub fn new(name: String, namespace: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            namespace,
            description,
            attributes: HashMap::new(),
            parent_entity: None,
            is_abstract: false,
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn add_attribute(&mut self, attribute: AttributeDefinition) {
        self.attributes.insert(attribute.id.clone(), attribute);
        self.updated_at = Utc::now();
    }

    pub fn set_parent(&mut self, parent_id: String) {
        self.parent_entity = Some(parent_id);
        self.updated_at = Utc::now();
    }

    pub fn set_abstract(&mut self, is_abstract: bool) {
        self.is_abstract = is_abstract;
        self.updated_at = Utc::now();
    }

    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }

    /// 获取所有属性（包括继承的属性）
    pub fn get_all_attributes(&self, repo: &MetaModelRepository) -> HashMap<String, AttributeDefinition> {
        let mut all_attributes = HashMap::new();
        
        // 递归获取父类属性
        if let Some(parent_id) = &self.parent_entity {
            if let Some(parent) = repo.entities.get(parent_id) {
                let parent_attrs = parent.get_all_attributes(repo);
                all_attributes.extend(parent_attrs);
            }
        }
        
        // 添加自身属性（会覆盖同名的父类属性）
        all_attributes.extend(self.attributes.clone());
        
        all_attributes
    }
}

/// 关系元模型 - 描述实体间的关联关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationMetaModel {
    pub id: String,
    pub name: String,
    pub relation_type: MetaRelationType,
    pub source_entity: String,
    pub target_entity: String,
    pub cardinality: Cardinality,
    pub description: Option<String>,
    pub is_navigable: bool,
    pub is_bidirectional: bool,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RelationMetaModel {
    pub fn new(
        name: String,
        relation_type: MetaRelationType,
        source_entity: String,
        target_entity: String,
        cardinality: Cardinality,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            relation_type,
            source_entity,
            target_entity,
            cardinality,
            description: None,
            is_navigable: true,
            is_bidirectional: false,
            version: "1.0.0".to_string(),
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    pub fn set_bidirectional(mut self, bidirectional: bool) -> Self {
        self.is_bidirectional = bidirectional;
        self
    }

    pub fn add_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 元模型仓库 - 管理所有实体和关系元模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaModelRepository {
    pub entities: HashMap<String, EntityMetaModel>,
    pub relations: HashMap<String, RelationMetaModel>,
    pub namespaces: HashMap<String, Vec<String>>, // namespace -> entity_ids
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MetaModelRepository {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            entities: HashMap::new(),
            relations: HashMap::new(),
            namespaces: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_entity_meta_model(&mut self, entity: EntityMetaModel) -> Result<(), String> {
        // 检查名称冲突
        if self.entities.values().any(|e| e.name == entity.name && e.namespace == entity.namespace) {
            return Err(format!("Entity '{}' already exists in namespace '{}'", entity.name, entity.namespace));
        }

        // 添加到命名空间索引
        self.namespaces
            .entry(entity.namespace.clone())
            .or_insert_with(Vec::new)
            .push(entity.id.clone());

        self.entities.insert(entity.id.clone(), entity);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_relation_meta_model(&mut self, relation: RelationMetaModel) -> Result<(), String> {
        // 验证源实体和目标实体存在
        if !self.entities.contains_key(&relation.source_entity) {
            return Err(format!("Source entity '{}' not found", relation.source_entity));
        }
        if !self.entities.contains_key(&relation.target_entity) {
            return Err(format!("Target entity '{}' not found", relation.target_entity));
        }

        self.relations.insert(relation.id.clone(), relation);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_entity_by_name(&self, name: &str, namespace: &str) -> Option<&EntityMetaModel> {
        self.entities.values().find(|e| e.name == name && e.namespace == namespace)
    }

    pub fn get_entities_in_namespace(&self, namespace: &str) -> Vec<&EntityMetaModel> {
        if let Some(entity_ids) = self.namespaces.get(namespace) {
            entity_ids.iter()
                .filter_map(|id| self.entities.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_relations_for_entity(&self, entity_id: &str) -> Vec<&RelationMetaModel> {
        self.relations.values()
            .filter(|r| r.source_entity == entity_id || r.target_entity == entity_id)
            .collect()
    }

    pub fn validate_model(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // 验证实体继承关系
        for entity in self.entities.values() {
            if let Some(parent_id) = &entity.parent_entity {
                if !self.entities.contains_key(parent_id) {
                    errors.push(format!("Entity '{}' references non-existent parent '{}'", entity.name, parent_id));
                }
            }
        }

        // 验证关系引用
        for relation in self.relations.values() {
            if !self.entities.contains_key(&relation.source_entity) {
                errors.push(format!("Relation '{}' references non-existent source entity '{}'", relation.name, relation.source_entity));
            }
            if !self.entities.contains_key(&relation.target_entity) {
                errors.push(format!("Relation '{}' references non-existent target entity '{}'", relation.name, relation.target_entity));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for MetaModelRepository {
    fn default() -> Self {
        Self::new()
    }
}
