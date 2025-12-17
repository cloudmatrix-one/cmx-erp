/// 华为iDME业务流程扩展模块
/// 
/// 提供业务实体生成器，支持会计凭证、销售订单等业务场景

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::{
    EntityMetaModel, RelationMetaModel, MetaModelRepository, AttributeDefinition,
    DataType, MetaRelationType, Cardinality
};

/// 业务流程模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessProcessTemplate {
    pub id: String,
    pub name: String,
    pub process_type: String,
    pub description: Option<String>,
    pub entity_ids: Vec<String>,
    pub relation_ids: Vec<String>,
    pub workflow_steps: Vec<WorkflowStep>,
    pub created_at: DateTime<Utc>,
}

/// 工作流步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub step_id: String,
    pub step_name: String,
    pub entity_type: String,
    pub operation: String,
    pub next_steps: Vec<String>,
    pub conditions: Option<HashMap<String, serde_json::Value>>,
}

/// 业务实体生成器
#[derive(Debug, Clone)]
pub struct BusinessEntityGenerator {
    pub meta_repo: MetaModelRepository,
}

impl BusinessEntityGenerator {
    pub fn new() -> Self {
        Self {
            meta_repo: MetaModelRepository::new(),
        }
    }

    /// 生成完整的销售业务流程模板
    pub fn generate_sales_business_template(&mut self) -> Result<BusinessProcessTemplate, String> {
        let mut entity_ids = Vec::new();
        let mut relation_ids = Vec::new();

        // 1. 生成客户实体
        let customer_id = self.generate_customer_entity()?;
        entity_ids.push(customer_id.clone());

        // 2. 生成销售订单实体
        let sales_order_id = self.generate_sales_order_entity()?;
        entity_ids.push(sales_order_id.clone());

        // 3. 生成订单明细实体
        let order_line_id = self.generate_sales_order_line_entity()?;
        entity_ids.push(order_line_id.clone());

        // 4. 生成销售发票实体
        let invoice_id = self.generate_sales_invoice_entity()?;
        entity_ids.push(invoice_id.clone());

        // 5. 生成会计凭证实体
        let voucher_id = self.generate_accounting_voucher_entity()?;
        entity_ids.push(voucher_id.clone());

        // 6. 生成凭证明细实体
        let voucher_line_id = self.generate_voucher_line_entity()?;
        entity_ids.push(voucher_line_id.clone());

        // 7. 生成关系
        // 客户 -> 销售订单 (一对多关联)
        let customer_order_rel = RelationMetaModel::new(
            "CustomerOrderAssociation".to_string(),
            MetaRelationType::Association,
            customer_id.clone(),
            sales_order_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("客户与销售订单的关联关系".to_string());
        relation_ids.push(customer_order_rel.id.clone());
        self.meta_repo.add_relation_meta_model(customer_order_rel)?;

        // 销售订单 -> 订单明细 (一对多组合)
        let order_line_rel = RelationMetaModel::new(
            "OrderLineComposition".to_string(),
            MetaRelationType::Composition,
            sales_order_id.clone(),
            order_line_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("销售订单与订单明细的组合关系".to_string());
        relation_ids.push(order_line_rel.id.clone());
        self.meta_repo.add_relation_meta_model(order_line_rel)?;

        // 销售订单 -> 销售发票 (一对多依赖)
        let order_invoice_rel = RelationMetaModel::new(
            "OrderInvoiceDependency".to_string(),
            MetaRelationType::Dependency,
            sales_order_id.clone(),
            invoice_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("销售订单与销售发票的依赖关系".to_string());
        relation_ids.push(order_invoice_rel.id.clone());
        self.meta_repo.add_relation_meta_model(order_invoice_rel)?;

        // 销售发票 -> 会计凭证 (一对多依赖)
        let invoice_voucher_rel = RelationMetaModel::new(
            "InvoiceVoucherDependency".to_string(),
            MetaRelationType::Dependency,
            invoice_id.clone(),
            voucher_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("销售发票与会计凭证的依赖关系".to_string());
        relation_ids.push(invoice_voucher_rel.id.clone());
        self.meta_repo.add_relation_meta_model(invoice_voucher_rel)?;

        // 会计凭证 -> 凭证明细 (一对多组合)
        let voucher_line_rel = RelationMetaModel::new(
            "VoucherLineComposition".to_string(),
            MetaRelationType::Composition,
            voucher_id.clone(),
            voucher_line_id.clone(),
            Cardinality::one_to_many(),
        ).with_description("会计凭证与凭证明细的组合关系".to_string());
        relation_ids.push(voucher_line_rel.id.clone());
        self.meta_repo.add_relation_meta_model(voucher_line_rel)?;

        // 8. 定义工作流步骤
        let workflow_steps = vec![
            WorkflowStep {
                step_id: "step_1".to_string(),
                step_name: "创建销售订单".to_string(),
                entity_type: "SalesOrder".to_string(),
                operation: "create".to_string(),
                next_steps: vec!["step_2".to_string()],
                conditions: None,
            },
            WorkflowStep {
                step_id: "step_2".to_string(),
                step_name: "生成销售发票".to_string(),
                entity_type: "SalesInvoice".to_string(),
                operation: "create".to_string(),
                next_steps: vec!["step_3".to_string()],
                conditions: Some({
                    let mut cond = HashMap::new();
                    cond.insert("order_status".to_string(), serde_json::Value::String("confirmed".to_string()));
                    cond
                }),
            },
            WorkflowStep {
                step_id: "step_3".to_string(),
                step_name: "生成会计凭证".to_string(),
                entity_type: "AccountingVoucher".to_string(),
                operation: "create".to_string(),
                next_steps: vec![],
                conditions: Some({
                    let mut cond = HashMap::new();
                    cond.insert("invoice_status".to_string(), serde_json::Value::String("issued".to_string()));
                    cond
                }),
            },
        ];

        Ok(BusinessProcessTemplate {
            id: Uuid::new_v4().to_string(),
            name: "销售业务流程模板".to_string(),
            process_type: "sales_process".to_string(),
            description: Some("完整的销售业务流程，包含订单、发票、凭证等环节".to_string()),
            entity_ids,
            relation_ids,
            workflow_steps,
            created_at: Utc::now(),
        })
    }

    /// 生成客户实体
    pub fn generate_customer_entity(&mut self) -> Result<String, String> {
        let mut customer_entity = EntityMetaModel::new(
            "Customer".to_string(),
            "business".to_string(),
            Some("客户实体".to_string()),
        );

        // 添加客户属性
        customer_entity.add_attribute(AttributeDefinition::new(
            "customer_code".to_string(),
            DataType::String,
            true,
        ).with_description("客户编码".to_string()));

        customer_entity.add_attribute(AttributeDefinition::new(
            "customer_name".to_string(),
            DataType::String,
            true,
        ).with_description("客户名称".to_string()));

        customer_entity.add_attribute(AttributeDefinition::new(
            "customer_type".to_string(),
            DataType::String,
            false,
        ).with_description("客户类型".to_string()));

        customer_entity.add_attribute(AttributeDefinition::new(
            "contact_person".to_string(),
            DataType::String,
            false,
        ).with_description("联系人".to_string()));

        customer_entity.add_attribute(AttributeDefinition::new(
            "phone".to_string(),
            DataType::String,
            false,
        ).with_description("联系电话".to_string()));

        customer_entity.add_attribute(AttributeDefinition::new(
            "email".to_string(),
            DataType::String,
            false,
        ).with_description("电子邮箱".to_string()));

        customer_entity.add_attribute(AttributeDefinition::new(
            "address".to_string(),
            DataType::Text,
            false,
        ).with_description("地址".to_string()));

        let entity_id = customer_entity.id.clone();
        self.meta_repo.add_entity_meta_model(customer_entity)?;
        Ok(entity_id)
    }

    /// 生成销售订单实体
    pub fn generate_sales_order_entity(&mut self) -> Result<String, String> {
        let mut sales_order_entity = EntityMetaModel::new(
            "SalesOrder".to_string(),
            "business".to_string(),
            Some("销售订单实体".to_string()),
        );

        // 添加销售订单属性
        sales_order_entity.add_attribute(AttributeDefinition::new(
            "order_number".to_string(),
            DataType::String,
            true,
        ).with_description("订单编号".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "order_date".to_string(),
            DataType::DateTime,
            true,
        ).with_description("订单日期".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "customer_id".to_string(),
            DataType::Reference("Customer".to_string()),
            false,
        ).with_description("客户ID".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "customer_name".to_string(),
            DataType::String,
            true,
        ).with_description("客户名称".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "total_amount".to_string(),
            DataType::Decimal,
            true,
        ).with_description("订单总金额".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "currency".to_string(),
            DataType::String,
            true,
        ).with_description("币种".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "status".to_string(),
            DataType::String,
            true,
        ).with_description("订单状态".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "delivery_date".to_string(),
            DataType::DateTime,
            false,
        ).with_description("交货日期".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "sales_person".to_string(),
            DataType::String,
            false,
        ).with_description("销售员".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "payment_terms".to_string(),
            DataType::String,
            false,
        ).with_description("付款条件".to_string()));

        sales_order_entity.add_attribute(AttributeDefinition::new(
            "notes".to_string(),
            DataType::Text,
            false,
        ).with_description("备注".to_string()));

        let entity_id = sales_order_entity.id.clone();
        self.meta_repo.add_entity_meta_model(sales_order_entity)?;
        Ok(entity_id)
    }

    /// 生成销售订单明细实体
    pub fn generate_sales_order_line_entity(&mut self) -> Result<String, String> {
        let mut order_line_entity = EntityMetaModel::new(
            "SalesOrderLine".to_string(),
            "business".to_string(),
            Some("销售订单明细实体".to_string()),
        );

        // 添加订单明细属性
        order_line_entity.add_attribute(AttributeDefinition::new(
            "line_number".to_string(),
            DataType::Integer,
            true,
        ).with_description("行号".to_string()));

        order_line_entity.add_attribute(AttributeDefinition::new(
            "product_id".to_string(),
            DataType::Reference("Product".to_string()),
            false,
        ).with_description("产品ID".to_string()));

        order_line_entity.add_attribute(AttributeDefinition::new(
            "product_name".to_string(),
            DataType::String,
            true,
        ).with_description("产品名称".to_string()));

        order_line_entity.add_attribute(AttributeDefinition::new(
            "quantity".to_string(),
            DataType::Decimal,
            true,
        ).with_description("数量".to_string()));

        order_line_entity.add_attribute(AttributeDefinition::new(
            "unit_price".to_string(),
            DataType::Decimal,
            true,
        ).with_description("单价".to_string()));

        order_line_entity.add_attribute(AttributeDefinition::new(
            "line_amount".to_string(),
            DataType::Decimal,
            true,
        ).with_description("行金额".to_string()));

        order_line_entity.add_attribute(AttributeDefinition::new(
            "discount_rate".to_string(),
            DataType::Decimal,
            false,
        ).with_description("折扣率".to_string()));

        order_line_entity.add_attribute(AttributeDefinition::new(
            "tax_rate".to_string(),
            DataType::Decimal,
            false,
        ).with_description("税率".to_string()));

        let entity_id = order_line_entity.id.clone();
        self.meta_repo.add_entity_meta_model(order_line_entity)?;
        Ok(entity_id)
    }

    /// 生成销售发票实体
    pub fn generate_sales_invoice_entity(&mut self) -> Result<String, String> {
        let mut invoice_entity = EntityMetaModel::new(
            "SalesInvoice".to_string(),
            "business".to_string(),
            Some("销售发票实体".to_string()),
        );

        // 添加销售发票属性
        invoice_entity.add_attribute(AttributeDefinition::new(
            "invoice_number".to_string(),
            DataType::String,
            true,
        ).with_description("发票编号".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "invoice_date".to_string(),
            DataType::DateTime,
            true,
        ).with_description("发票日期".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "sales_order_id".to_string(),
            DataType::Reference("SalesOrder".to_string()),
            false,
        ).with_description("销售订单ID".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "customer_id".to_string(),
            DataType::Reference("Customer".to_string()),
            false,
        ).with_description("客户ID".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "customer_name".to_string(),
            DataType::String,
            true,
        ).with_description("客户名称".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "subtotal_amount".to_string(),
            DataType::Decimal,
            true,
        ).with_description("小计金额".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "tax_amount".to_string(),
            DataType::Decimal,
            true,
        ).with_description("税额".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "total_amount".to_string(),
            DataType::Decimal,
            true,
        ).with_description("总金额".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "currency".to_string(),
            DataType::String,
            true,
        ).with_description("币种".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "status".to_string(),
            DataType::String,
            true,
        ).with_description("发票状态".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "due_date".to_string(),
            DataType::DateTime,
            false,
        ).with_description("到期日期".to_string()));

        invoice_entity.add_attribute(AttributeDefinition::new(
            "payment_status".to_string(),
            DataType::String,
            false,
        ).with_description("付款状态".to_string()));

        let entity_id = invoice_entity.id.clone();
        self.meta_repo.add_entity_meta_model(invoice_entity)?;
        Ok(entity_id)
    }

    /// 生成会计凭证实体
    pub fn generate_accounting_voucher_entity(&mut self) -> Result<String, String> {
        let mut voucher_entity = EntityMetaModel::new(
            "AccountingVoucher".to_string(),
            "business".to_string(),
            Some("会计凭证实体".to_string()),
        );

        // 添加会计凭证属性
        voucher_entity.add_attribute(AttributeDefinition::new(
            "voucher_number".to_string(),
            DataType::String,
            true,
        ).with_description("凭证编号".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "voucher_date".to_string(),
            DataType::DateTime,
            true,
        ).with_description("凭证日期".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "voucher_type".to_string(),
            DataType::String,
            true,
        ).with_description("凭证类型".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "reference_document_type".to_string(),
            DataType::String,
            false,
        ).with_description("参考单据类型".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "reference_document_id".to_string(),
            DataType::String,
            false,
        ).with_description("参考单据ID".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "total_debit_amount".to_string(),
            DataType::Decimal,
            true,
        ).with_description("借方总金额".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "total_credit_amount".to_string(),
            DataType::Decimal,
            true,
        ).with_description("贷方总金额".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "currency".to_string(),
            DataType::String,
            true,
        ).with_description("币种".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "status".to_string(),
            DataType::String,
            true,
        ).with_description("凭证状态".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "prepared_by".to_string(),
            DataType::String,
            false,
        ).with_description("制单人".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "approved_by".to_string(),
            DataType::String,
            false,
        ).with_description("审核人".to_string()));

        voucher_entity.add_attribute(AttributeDefinition::new(
            "description".to_string(),
            DataType::Text,
            false,
        ).with_description("摘要".to_string()));

        let entity_id = voucher_entity.id.clone();
        self.meta_repo.add_entity_meta_model(voucher_entity)?;
        Ok(entity_id)
    }

    /// 生成凭证明细实体
    pub fn generate_voucher_line_entity(&mut self) -> Result<String, String> {
        let mut voucher_line_entity = EntityMetaModel::new(
            "VoucherLine".to_string(),
            "business".to_string(),
            Some("凭证明细实体".to_string()),
        );

        // 添加凭证明细属性
        voucher_line_entity.add_attribute(AttributeDefinition::new(
            "line_number".to_string(),
            DataType::Integer,
            true,
        ).with_description("行号".to_string()));

        voucher_line_entity.add_attribute(AttributeDefinition::new(
            "account_code".to_string(),
            DataType::String,
            true,
        ).with_description("科目代码".to_string()));

        voucher_line_entity.add_attribute(AttributeDefinition::new(
            "account_name".to_string(),
            DataType::String,
            true,
        ).with_description("科目名称".to_string()));

        voucher_line_entity.add_attribute(AttributeDefinition::new(
            "debit_amount".to_string(),
            DataType::Decimal,
            false,
        ).with_description("借方金额".to_string()));

        voucher_line_entity.add_attribute(AttributeDefinition::new(
            "credit_amount".to_string(),
            DataType::Decimal,
            false,
        ).with_description("贷方金额".to_string()));

        voucher_line_entity.add_attribute(AttributeDefinition::new(
            "description".to_string(),
            DataType::Text,
            false,
        ).with_description("摘要".to_string()));

        voucher_line_entity.add_attribute(AttributeDefinition::new(
            "cost_center".to_string(),
            DataType::String,
            false,
        ).with_description("成本中心".to_string()));

        voucher_line_entity.add_attribute(AttributeDefinition::new(
            "project_id".to_string(),
            DataType::Reference("Project".to_string()),
            false,
        ).with_description("项目ID".to_string()));

        let entity_id = voucher_line_entity.id.clone();
        self.meta_repo.add_entity_meta_model(voucher_line_entity)?;
        Ok(entity_id)
    }

    /// 生成通用主从结构业务实体模板
    pub fn generate_master_detail_template(
        &mut self,
        master_name: String,
        detail_name: String,
        namespace: String,
        master_attributes: Vec<(String, DataType, bool)>, // (name, type, required)
        detail_attributes: Vec<(String, DataType, bool)>,
    ) -> Result<BusinessProcessTemplate, String> {
        let mut entity_ids = Vec::new();
        let mut relation_ids = Vec::new();

        // 1. 生成主表实体
        let mut master_entity = EntityMetaModel::new(
            master_name.clone(),
            namespace.clone(),
            Some(format!("{}主表实体", master_name)),
        );

        for (attr_name, data_type, is_required) in master_attributes {
            master_entity.add_attribute(AttributeDefinition::new(
                attr_name.clone(),
                data_type,
                is_required,
            ).with_description(format!("{}属性", attr_name)));
        }

        let master_id = master_entity.id.clone();
        entity_ids.push(master_id.clone());
        self.meta_repo.add_entity_meta_model(master_entity)?;

        // 2. 生成从表实体
        let mut detail_entity = EntityMetaModel::new(
            detail_name.clone(),
            namespace.clone(),
            Some(format!("{}明细实体", detail_name)),
        );

        for (attr_name, data_type, is_required) in detail_attributes {
            detail_entity.add_attribute(AttributeDefinition::new(
                attr_name.clone(),
                data_type,
                is_required,
            ).with_description(format!("{}属性", attr_name)));
        }

        let detail_id = detail_entity.id.clone();
        entity_ids.push(detail_id.clone());
        self.meta_repo.add_entity_meta_model(detail_entity)?;

        // 3. 生成主从关系（组合关系）
        let master_detail_rel = RelationMetaModel::new(
            format!("{}{}Composition", master_name, detail_name),
            MetaRelationType::Composition,
            master_id.clone(),
            detail_id.clone(),
            Cardinality::one_to_many(),
        ).with_description(format!("{}与{}的组合关系", master_name, detail_name));

        relation_ids.push(master_detail_rel.id.clone());
        self.meta_repo.add_relation_meta_model(master_detail_rel)?;

        // 4. 定义工作流步骤
        let workflow_steps = vec![
            WorkflowStep {
                step_id: "step_1".to_string(),
                step_name: format!("创建{}", master_name),
                entity_type: master_name.clone(),
                operation: "create".to_string(),
                next_steps: vec!["step_2".to_string()],
                conditions: None,
            },
            WorkflowStep {
                step_id: "step_2".to_string(),
                step_name: format!("添加{}明细", detail_name),
                entity_type: detail_name.clone(),
                operation: "create".to_string(),
                next_steps: vec![],
                conditions: None,
            },
        ];

        Ok(BusinessProcessTemplate {
            id: Uuid::new_v4().to_string(),
            name: format!("{}业务流程模板", master_name),
            process_type: "master_detail_process".to_string(),
            description: Some(format!("通用的{}主从结构业务流程", master_name)),
            entity_ids,
            relation_ids,
            workflow_steps,
            created_at: Utc::now(),
        })
    }
}

impl Default for BusinessEntityGenerator {
    fn default() -> Self {
        Self::new()
    }
}
