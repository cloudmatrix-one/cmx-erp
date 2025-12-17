/// 华为iDME核心实例管理模块
/// 
/// 提供实体实例的创建、管理和操作功能

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{EntityMetaModel, MetaModelRepository, DataType};

/// 实体实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInstance {
    pub id: String,
    pub entity_meta_id: String,
    pub attributes: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
    pub status: InstanceStatus,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// 实例状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstanceStatus {
    Draft,      // 草稿
    Active,     // 活跃
    Inactive,   // 非活跃
    Archived,   // 已归档
    Deleted,    // 已删除
}

impl EntityInstance {
    pub fn new(entity_meta_id: String, attributes: HashMap<String, serde_json::Value>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            entity_meta_id,
            attributes,
            created_at: now,
            updated_at: now,
            version: 1,
            status: InstanceStatus::Draft,
            metadata: HashMap::new(),
        }
    }

    pub fn set_attribute(&mut self, key: String, value: serde_json::Value) {
        self.attributes.insert(key, value);
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn get_attribute(&self, key: &str) -> Option<&serde_json::Value> {
        self.attributes.get(key)
    }

    pub fn set_status(&mut self, status: InstanceStatus) {
        self.status = status;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
        self.updated_at = Utc::now();
    }
}

/// 关系实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationInstance {
    pub id: String,
    pub relation_meta_id: String,
    pub source_instance_id: String,
    pub target_instance_id: String,
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: InstanceStatus,
}

impl RelationInstance {
    pub fn new(
        relation_meta_id: String,
        source_instance_id: String,
        target_instance_id: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            relation_meta_id,
            source_instance_id,
            target_instance_id,
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
            status: InstanceStatus::Active,
        }
    }

    pub fn set_property(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
        self.updated_at = Utc::now();
    }
}

/// 实例管理器
#[derive(Debug, Clone)]
pub struct InstanceManager {
    pub meta_repo: MetaModelRepository,
    pub entities: HashMap<String, EntityInstance>,
    pub relations: HashMap<String, RelationInstance>,
    pub entity_index: HashMap<String, Vec<String>>, // entity_meta_id -> instance_ids
}

impl InstanceManager {
    pub fn new(meta_repo: MetaModelRepository) -> Self {
        Self {
            meta_repo,
            entities: HashMap::new(),
            relations: HashMap::new(),
            entity_index: HashMap::new(),
        }
    }

    /// 创建实体实例
    pub fn create_entity_instance(
        &mut self,
        entity_meta_id: String,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<String, String> {
        // 验证元模型存在
        let entity_meta = self.meta_repo.entities.get(&entity_meta_id)
            .ok_or_else(|| format!("Entity meta model '{}' not found", entity_meta_id))?;

        // 验证属性
        self.validate_attributes(entity_meta, &attributes)?;

        // 创建实例
        let instance = EntityInstance::new(entity_meta_id.clone(), attributes);
        let instance_id = instance.id.clone();

        // 添加到索引
        self.entity_index
            .entry(entity_meta_id)
            .or_insert_with(Vec::new)
            .push(instance_id.clone());

        self.entities.insert(instance_id.clone(), instance);
        Ok(instance_id)
    }

    /// 创建关系实例
    pub fn create_relation_instance(
        &mut self,
        relation_meta_id: String,
        source_instance_id: String,
        target_instance_id: String,
    ) -> Result<String, String> {
        // 验证关系元模型存在
        let _relation_meta = self.meta_repo.relations.get(&relation_meta_id)
            .ok_or_else(|| format!("Relation meta model '{}' not found", relation_meta_id))?;

        // 验证实例存在
        if !self.entities.contains_key(&source_instance_id) {
            return Err(format!("Source instance '{}' not found", source_instance_id));
        }
        if !self.entities.contains_key(&target_instance_id) {
            return Err(format!("Target instance '{}' not found", target_instance_id));
        }

        // 创建关系实例
        let relation_instance = RelationInstance::new(
            relation_meta_id,
            source_instance_id,
            target_instance_id,
        );
        let relation_id = relation_instance.id.clone();

        self.relations.insert(relation_id.clone(), relation_instance);
        Ok(relation_id)
    }

    /// 获取实体实例
    pub fn get_entity_instance(&self, instance_id: &str) -> Option<&EntityInstance> {
        self.entities.get(instance_id)
    }

    /// 获取实体实例（可变引用）
    pub fn get_entity_instance_mut(&mut self, instance_id: &str) -> Option<&mut EntityInstance> {
        self.entities.get_mut(instance_id)
    }

    /// 根据元模型ID获取所有实例
    pub fn get_instances_by_meta_id(&self, entity_meta_id: &str) -> Vec<&EntityInstance> {
        if let Some(instance_ids) = self.entity_index.get(entity_meta_id) {
            instance_ids.iter()
                .filter_map(|id| self.entities.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 根据属性查询实例
    pub fn query_instances_by_attribute(
        &self,
        entity_meta_id: &str,
        attribute_name: &str,
        value: &serde_json::Value,
    ) -> Vec<&EntityInstance> {
        self.get_instances_by_meta_id(entity_meta_id)
            .into_iter()
            .filter(|instance| {
                instance.get_attribute(attribute_name)
                    .map_or(false, |v| v == value)
            })
            .collect()
    }

    /// 更新实体实例
    pub fn update_entity_instance(
        &mut self,
        instance_id: &str,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        // 先获取实例的元模型ID
        let entity_meta_id = {
            let instance = self.entities.get(instance_id)
                .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;
            instance.entity_meta_id.clone()
        };

        // 验证属性
        let entity_meta = self.meta_repo.entities.get(&entity_meta_id)
            .ok_or_else(|| format!("Entity meta model '{}' not found", entity_meta_id))?;

        self.validate_attributes(entity_meta, &attributes)?;

        // 更新属性
        let instance = self.entities.get_mut(instance_id)
            .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;

        for (key, value) in attributes {
            instance.set_attribute(key, value);
        }

        Ok(())
    }

    /// 删除实体实例
    pub fn delete_entity_instance(&mut self, instance_id: &str) -> Result<(), String> {
        let instance = self.entities.get(instance_id)
            .ok_or_else(|| format!("Instance '{}' not found", instance_id))?;

        let entity_meta_id = instance.entity_meta_id.clone();

        // 从索引中移除
        if let Some(instance_ids) = self.entity_index.get_mut(&entity_meta_id) {
            instance_ids.retain(|id| id != instance_id);
        }

        // 删除相关的关系实例
        let related_relations: Vec<String> = self.relations.values()
            .filter(|r| r.source_instance_id == instance_id || r.target_instance_id == instance_id)
            .map(|r| r.id.clone())
            .collect();

        for relation_id in related_relations {
            self.relations.remove(&relation_id);
        }

        // 删除实例
        self.entities.remove(instance_id);
        Ok(())
    }

    /// 验证属性
    fn validate_attributes(
        &self,
        entity_meta: &EntityMetaModel,
        attributes: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        let all_attributes = entity_meta.get_all_attributes(&self.meta_repo);

        // 检查必需属性
        for (attr_id, attr_def) in &all_attributes {
            if attr_def.is_required && !attributes.contains_key(attr_id) && !attributes.contains_key(&attr_def.name) {
                return Err(format!("Required attribute '{}' is missing", attr_def.name));
            }
        }

        // 验证属性类型
        for (key, value) in attributes {
            // 通过ID或名称查找属性定义
            let attr_def = all_attributes.get(key)
                .or_else(|| all_attributes.values().find(|attr| attr.name == *key));

            if let Some(attr_def) = attr_def {
                if !self.validate_value_type(value, &attr_def.data_type) {
                    return Err(format!("Invalid type for attribute '{}': expected {:?}", attr_def.name, attr_def.data_type));
                }
            }
        }

        Ok(())
    }

    /// 验证值类型
    fn validate_value_type(&self, value: &serde_json::Value, expected_type: &DataType) -> bool {
        match (value, expected_type) {
            (serde_json::Value::String(_), DataType::String) => true,
            (serde_json::Value::String(_), DataType::Text) => true,
            (serde_json::Value::Number(n), DataType::Integer) => n.is_i64(),
            (serde_json::Value::Number(n), DataType::Float) => n.is_f64(),
            (serde_json::Value::Number(_), DataType::Decimal) => true,
            (serde_json::Value::Bool(_), DataType::Boolean) => true,
            (serde_json::Value::String(_), DataType::DateTime) => {
                // 简单验证：检查是否为有效的ISO 8601格式
                value.as_str().map_or(false, |s| chrono::DateTime::parse_from_rfc3339(s).is_ok())
            },
            (serde_json::Value::Object(_), DataType::Json) => true,
            (serde_json::Value::Array(_), DataType::Json) => true,
            (serde_json::Value::String(_), DataType::Reference(_)) => true,
            (serde_json::Value::Null, _) => true, // null值总是允许的
            _ => false,
        }
    }

    /// 获取实例统计信息
    pub fn get_statistics(&self) -> InstanceStatistics {
        let mut entity_counts = HashMap::new();
        for instance in self.entities.values() {
            *entity_counts.entry(instance.entity_meta_id.clone()).or_insert(0) += 1;
        }

        InstanceStatistics {
            total_entities: self.entities.len(),
            total_relations: self.relations.len(),
            entity_counts,
            created_at: Utc::now(),
        }
    }
}

/// 实例统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct InstanceStatistics {
    pub total_entities: usize,
    pub total_relations: usize,
    pub entity_counts: HashMap<String, usize>, // entity_meta_id -> count
    pub created_at: DateTime<Utc>,
}
