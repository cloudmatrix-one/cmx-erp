/// 华为iDME工业领域扩展模块
///
/// 提供工业制造领域的专用实体和模板，如BOM管理、产品结构等

use uuid::Uuid;
use chrono::Utc;

use super::{
    EntityMetaModel, RelationMetaModel, MetaModelRepository, AttributeDefinition,
    DataType, MetaRelationType, Cardinality, BusinessProcessTemplate, WorkflowStep
};

/// 工业BOM模板生成器
#[derive(Debug, Clone)]
pub struct IndustrialBOMGenerator {
    pub meta_repo: MetaModelRepository,
}

impl IndustrialBOMGenerator {
    pub fn new() -> Self {
        Self {
            meta_repo: MetaModelRepository::new(),
        }
    }

    /// 生成完整的BOM管理模板
    pub fn generate_bom_management_template(&mut self) -> Result<BusinessProcessTemplate, String> {
        let mut entity_ids = Vec::new();
        let mut relation_ids = Vec::new();

        // 1. 生成产品实体
        let product_id = self.generate_product_entity()?;
        entity_ids.push(product_id.clone());

        // 2. 生成物料实体
        let material_id = self.generate_material_entity()?;
        entity_ids.push(material_id.clone());

        // 3. 生成BOM头实体
        let bom_header_id = self.generate_bom_header_entity()?;
        entity_ids.push(bom_header_id.clone());

        // 4. 生成BOM明细实体
        let bom_line_id = self.generate_bom_line_entity()?;
        entity_ids.push(bom_line_id.clone());

        // 5. 生成工艺路线实体
        let routing_id = self.generate_routing_entity()?;
        entity_ids.push(routing_id.clone());

        // 6. 生成工序实体
        let operation_id = self.generate_operation_entity()?;
        entity_ids.push(operation_id.clone());

        // 7. 生成关系
        // 产品 -> BOM头 (一对多关联)
        let product_bom_rel = RelationMetaModel::new(
            "ProductBOMAssociation".to_string(),
            MetaRelationType::Association,
            product_id.clone(),
            bom_header_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("产品与BOM的关联关系".to_string());
        relation_ids.push(product_bom_rel.id.clone());
        self.meta_repo.add_relation_meta_model(product_bom_rel)?;

        // BOM头 -> BOM明细 (一对多组合)
        let bom_line_rel = RelationMetaModel::new(
            "BOMLineComposition".to_string(),
            MetaRelationType::Composition,
            bom_header_id.clone(),
            bom_line_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("BOM头与BOM明细的组合关系".to_string());
        relation_ids.push(bom_line_rel.id.clone());
        self.meta_repo.add_relation_meta_model(bom_line_rel)?;

        // 物料 -> BOM明细 (一对多关联)
        let material_bom_rel = RelationMetaModel::new(
            "MaterialBOMAssociation".to_string(),
            MetaRelationType::Association,
            material_id.clone(),
            bom_line_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("物料与BOM明细的关联关系".to_string());
        relation_ids.push(material_bom_rel.id.clone());
        self.meta_repo.add_relation_meta_model(material_bom_rel)?;

        // 产品 -> 工艺路线 (一对多关联)
        let product_routing_rel = RelationMetaModel::new(
            "ProductRoutingAssociation".to_string(),
            MetaRelationType::Association,
            product_id.clone(),
            routing_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("产品与工艺路线的关联关系".to_string());
        relation_ids.push(product_routing_rel.id.clone());
        self.meta_repo.add_relation_meta_model(product_routing_rel)?;

        // 工艺路线 -> 工序 (一对多组合)
        let routing_operation_rel = RelationMetaModel::new(
            "RoutingOperationComposition".to_string(),
            MetaRelationType::Composition,
            routing_id.clone(),
            operation_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("工艺路线与工序的组合关系".to_string());
        relation_ids.push(routing_operation_rel.id.clone());
        self.meta_repo.add_relation_meta_model(routing_operation_rel)?;

        // 8. 定义工作流步骤
        let workflow_steps = vec![
            WorkflowStep {
                step_id: "step_1".to_string(),
                step_name: "创建产品".to_string(),
                entity_type: "Product".to_string(),
                operation: "create".to_string(),
                next_steps: vec!["step_2".to_string()],
                conditions: None,
            },
            WorkflowStep {
                step_id: "step_2".to_string(),
                step_name: "创建BOM结构".to_string(),
                entity_type: "BOMHeader".to_string(),
                operation: "create".to_string(),
                next_steps: vec!["step_3".to_string()],
                conditions: None,
            },
            WorkflowStep {
                step_id: "step_3".to_string(),
                step_name: "添加BOM明细".to_string(),
                entity_type: "BOMLine".to_string(),
                operation: "create".to_string(),
                next_steps: vec!["step_4".to_string()],
                conditions: None,
            },
            WorkflowStep {
                step_id: "step_4".to_string(),
                step_name: "定义工艺路线".to_string(),
                entity_type: "Routing".to_string(),
                operation: "create".to_string(),
                next_steps: vec!["step_5".to_string()],
                conditions: None,
            },
            WorkflowStep {
                step_id: "step_5".to_string(),
                step_name: "配置工序".to_string(),
                entity_type: "Operation".to_string(),
                operation: "create".to_string(),
                next_steps: vec![],
                conditions: None,
            },
        ];

        Ok(BusinessProcessTemplate {
            id: Uuid::new_v4().to_string(),
            name: "BOM管理流程模板".to_string(),
            process_type: "bom_management".to_string(),
            description: Some("完整的BOM管理流程，包含产品、物料、BOM结构、工艺路线等".to_string()),
            entity_ids,
            relation_ids,
            workflow_steps,
            created_at: Utc::now(),
        })
    }

    /// 生成产品实体
    pub fn generate_product_entity(&mut self) -> Result<String, String> {
        let mut product_entity = EntityMetaModel::new(
            "Product".to_string(),
            "manufacturing".to_string(),
            Some("产品实体".to_string()),
        );

        // 添加产品属性
        product_entity.add_attribute(AttributeDefinition::new(
            "product_code".to_string(),
            DataType::String,
            true,
        ).with_description("产品编码".to_string()));

        product_entity.add_attribute(AttributeDefinition::new(
            "product_name".to_string(),
            DataType::String,
            true,
        ).with_description("产品名称".to_string()));

        product_entity.add_attribute(AttributeDefinition::new(
            "product_type".to_string(),
            DataType::String,
            true,
        ).with_description("产品类型".to_string()));

        product_entity.add_attribute(AttributeDefinition::new(
            "specification".to_string(),
            DataType::Text,
            false,
        ).with_description("规格型号".to_string()));

        product_entity.add_attribute(AttributeDefinition::new(
            "unit_of_measure".to_string(),
            DataType::String,
            true,
        ).with_description("计量单位".to_string()));

        product_entity.add_attribute(AttributeDefinition::new(
            "standard_cost".to_string(),
            DataType::Decimal,
            false,
        ).with_description("标准成本".to_string()));

        product_entity.add_attribute(AttributeDefinition::new(
            "lead_time".to_string(),
            DataType::Integer,
            false,
        ).with_description("提前期（天）".to_string()));

        product_entity.add_attribute(AttributeDefinition::new(
            "status".to_string(),
            DataType::String,
            true,
        ).with_description("状态".to_string()));

        product_entity.add_attribute(AttributeDefinition::new(
            "created_date".to_string(),
            DataType::DateTime,
            true,
        ).with_description("创建日期".to_string()));

        let entity_id = product_entity.id.clone();
        self.meta_repo.add_entity_meta_model(product_entity)?;
        Ok(entity_id)
    }

    /// 生成物料实体
    pub fn generate_material_entity(&mut self) -> Result<String, String> {
        let mut material_entity = EntityMetaModel::new(
            "Material".to_string(),
            "manufacturing".to_string(),
            Some("物料实体".to_string()),
        );

        // 添加物料属性
        material_entity.add_attribute(AttributeDefinition::new(
            "material_code".to_string(),
            DataType::String,
            true,
        ).with_description("物料编码".to_string()));

        material_entity.add_attribute(AttributeDefinition::new(
            "material_name".to_string(),
            DataType::String,
            true,
        ).with_description("物料名称".to_string()));

        material_entity.add_attribute(AttributeDefinition::new(
            "material_type".to_string(),
            DataType::String,
            true,
        ).with_description("物料类型".to_string()));

        material_entity.add_attribute(AttributeDefinition::new(
            "specification".to_string(),
            DataType::Text,
            false,
        ).with_description("规格型号".to_string()));

        material_entity.add_attribute(AttributeDefinition::new(
            "unit_of_measure".to_string(),
            DataType::String,
            true,
        ).with_description("计量单位".to_string()));

        material_entity.add_attribute(AttributeDefinition::new(
            "unit_cost".to_string(),
            DataType::Decimal,
            false,
        ).with_description("单位成本".to_string()));

        material_entity.add_attribute(AttributeDefinition::new(
            "supplier_id".to_string(),
            DataType::Reference("Supplier".to_string()),
            false,
        ).with_description("供应商ID".to_string()));

        material_entity.add_attribute(AttributeDefinition::new(
            "status".to_string(),
            DataType::String,
            true,
        ).with_description("状态".to_string()));

        let entity_id = material_entity.id.clone();
        self.meta_repo.add_entity_meta_model(material_entity)?;
        Ok(entity_id)
    }

    /// 生成BOM头实体
    pub fn generate_bom_header_entity(&mut self) -> Result<String, String> {
        let mut bom_header_entity = EntityMetaModel::new(
            "BOMHeader".to_string(),
            "manufacturing".to_string(),
            Some("BOM头实体".to_string()),
        );

        // 添加BOM头属性
        bom_header_entity.add_attribute(AttributeDefinition::new(
            "bom_number".to_string(),
            DataType::String,
            true,
        ).with_description("BOM编号".to_string()));

        bom_header_entity.add_attribute(AttributeDefinition::new(
            "product_id".to_string(),
            DataType::Reference("Product".to_string()),
            true,
        ).with_description("产品ID".to_string()));

        bom_header_entity.add_attribute(AttributeDefinition::new(
            "version".to_string(),
            DataType::String,
            true,
        ).with_description("版本号".to_string()));

        bom_header_entity.add_attribute(AttributeDefinition::new(
            "effective_date".to_string(),
            DataType::DateTime,
            true,
        ).with_description("生效日期".to_string()));

        bom_header_entity.add_attribute(AttributeDefinition::new(
            "expiry_date".to_string(),
            DataType::DateTime,
            false,
        ).with_description("失效日期".to_string()));

        bom_header_entity.add_attribute(AttributeDefinition::new(
            "status".to_string(),
            DataType::String,
            true,
        ).with_description("状态".to_string()));

        bom_header_entity.add_attribute(AttributeDefinition::new(
            "description".to_string(),
            DataType::Text,
            false,
        ).with_description("描述".to_string()));

        let entity_id = bom_header_entity.id.clone();
        self.meta_repo.add_entity_meta_model(bom_header_entity)?;
        Ok(entity_id)
    }

    /// 生成BOM明细实体
    pub fn generate_bom_line_entity(&mut self) -> Result<String, String> {
        let mut bom_line_entity = EntityMetaModel::new(
            "BOMLine".to_string(),
            "manufacturing".to_string(),
            Some("BOM明细实体".to_string()),
        );

        // 添加BOM明细属性
        bom_line_entity.add_attribute(AttributeDefinition::new(
            "line_number".to_string(),
            DataType::Integer,
            true,
        ).with_description("行号".to_string()));

        bom_line_entity.add_attribute(AttributeDefinition::new(
            "material_id".to_string(),
            DataType::Reference("Material".to_string()),
            true,
        ).with_description("物料ID".to_string()));

        bom_line_entity.add_attribute(AttributeDefinition::new(
            "quantity".to_string(),
            DataType::Decimal,
            true,
        ).with_description("用量".to_string()));

        bom_line_entity.add_attribute(AttributeDefinition::new(
            "unit_of_measure".to_string(),
            DataType::String,
            true,
        ).with_description("计量单位".to_string()));

        bom_line_entity.add_attribute(AttributeDefinition::new(
            "scrap_factor".to_string(),
            DataType::Decimal,
            false,
        ).with_description("损耗率".to_string()));

        bom_line_entity.add_attribute(AttributeDefinition::new(
            "component_type".to_string(),
            DataType::String,
            false,
        ).with_description("组件类型".to_string()));

        bom_line_entity.add_attribute(AttributeDefinition::new(
            "position".to_string(),
            DataType::String,
            false,
        ).with_description("位置".to_string()));

        let entity_id = bom_line_entity.id.clone();
        self.meta_repo.add_entity_meta_model(bom_line_entity)?;
        Ok(entity_id)
    }

    /// 生成工艺路线实体
    pub fn generate_routing_entity(&mut self) -> Result<String, String> {
        let mut routing_entity = EntityMetaModel::new(
            "Routing".to_string(),
            "manufacturing".to_string(),
            Some("工艺路线实体".to_string()),
        );

        // 添加工艺路线属性
        routing_entity.add_attribute(AttributeDefinition::new(
            "routing_number".to_string(),
            DataType::String,
            true,
        ).with_description("工艺路线编号".to_string()));

        routing_entity.add_attribute(AttributeDefinition::new(
            "product_id".to_string(),
            DataType::Reference("Product".to_string()),
            true,
        ).with_description("产品ID".to_string()));

        routing_entity.add_attribute(AttributeDefinition::new(
            "version".to_string(),
            DataType::String,
            true,
        ).with_description("版本号".to_string()));

        routing_entity.add_attribute(AttributeDefinition::new(
            "effective_date".to_string(),
            DataType::DateTime,
            true,
        ).with_description("生效日期".to_string()));

        routing_entity.add_attribute(AttributeDefinition::new(
            "status".to_string(),
            DataType::String,
            true,
        ).with_description("状态".to_string()));

        routing_entity.add_attribute(AttributeDefinition::new(
            "description".to_string(),
            DataType::Text,
            false,
        ).with_description("描述".to_string()));

        let entity_id = routing_entity.id.clone();
        self.meta_repo.add_entity_meta_model(routing_entity)?;
        Ok(entity_id)
    }

    /// 生成工序实体
    pub fn generate_operation_entity(&mut self) -> Result<String, String> {
        let mut operation_entity = EntityMetaModel::new(
            "Operation".to_string(),
            "manufacturing".to_string(),
            Some("工序实体".to_string()),
        );

        // 添加工序属性
        operation_entity.add_attribute(AttributeDefinition::new(
            "operation_number".to_string(),
            DataType::String,
            true,
        ).with_description("工序编号".to_string()));

        operation_entity.add_attribute(AttributeDefinition::new(
            "operation_name".to_string(),
            DataType::String,
            true,
        ).with_description("工序名称".to_string()));

        operation_entity.add_attribute(AttributeDefinition::new(
            "sequence".to_string(),
            DataType::Integer,
            true,
        ).with_description("工序顺序".to_string()));

        operation_entity.add_attribute(AttributeDefinition::new(
            "work_center_id".to_string(),
            DataType::Reference("WorkCenter".to_string()),
            false,
        ).with_description("工作中心ID".to_string()));

        operation_entity.add_attribute(AttributeDefinition::new(
            "setup_time".to_string(),
            DataType::Decimal,
            false,
        ).with_description("准备时间（分钟）".to_string()));

        operation_entity.add_attribute(AttributeDefinition::new(
            "run_time".to_string(),
            DataType::Decimal,
            false,
        ).with_description("运行时间（分钟）".to_string()));

        operation_entity.add_attribute(AttributeDefinition::new(
            "description".to_string(),
            DataType::Text,
            false,
        ).with_description("工序描述".to_string()));

        let entity_id = operation_entity.id.clone();
        self.meta_repo.add_entity_meta_model(operation_entity)?;
        Ok(entity_id)
    }
}

impl Default for IndustrialBOMGenerator {
    fn default() -> Self {
        Self::new()
    }
}
