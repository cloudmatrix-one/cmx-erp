use sea_orm::{
    DatabaseConnection, DbErr, EntityTrait, QueryFilter, QuerySelect,
    TransactionTrait, DatabaseTransaction, QueryResult, FromQueryResult,
    Condition, Value, JsonValue, QueryBuilder,
};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use super::{DataSet, TableSchema, RowSet, CellValue, DataSetError};

// 表关系定义实体
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableRelation {
    pub parent_table: String,
    pub child_table: String,
    pub child_key: String,
    pub parent_key: String,
    pub relation_name: String,
}

#[async_trait]
pub trait SeaOrmDataSetStorage {
    async fn load_dataset(
        &self,
        schema: &TableSchema,
        query: &str,
        params: &HashMap<String, Value>,
    ) -> Result<DataSet, DataSetError>;

    async fn save_dataset(
        &self,
        schema: &TableSchema,
        dataset: &DataSet,
        parent_ref: Option<(&str, &Value)>,
    ) -> Result<(), DataSetError>;
}

pub struct SeaOrmStorage {
    db: DatabaseConnection,
}

impl SeaOrmStorage {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    // 将 SeaORM 查询结果转换为 CellValue
    fn convert_to_cell_value(value: &Value) -> Result<CellValue, DataSetError> {
        match value {
            Value::Int(Some(v)) => Ok(CellValue::Integer(*v as i32)),
            Value::BigInt(Some(v)) => Ok(CellValue::BigInt(*v)),
            Value::Float(Some(v)) => Ok(CellValue::Float(*v)),
            Value::Double(Some(v)) => Ok(CellValue::Double(*v)),
            Value::String(Some(v)) => Ok(CellValue::Text(Some(v.clone()))),
            Value::Bool(Some(v)) => Ok(CellValue::Boolean(*v)),
            Value::ChronoDate(Some(v)) => Ok(CellValue::Date(*v)),
            Value::ChronoTime(Some(v)) => Ok(CellValue::Time(*v)),
            Value::ChronoDateTime(Some(v)) => Ok(CellValue::Timestamp(*v)),
            Value::Null => Ok(CellValue::Null),
            _ => Err(DataSetError::TypeConversionError),
        }
    }

    // 将 CellValue 转换为 SeaORM Value
    fn convert_to_sea_value(value: &CellValue) -> Value {
        match value {
            CellValue::Null => Value::Null,
            CellValue::Integer(v) => Value::Int(Some(*v as i64)),
            CellValue::BigInt(v) => Value::BigInt(Some(*v)),
            CellValue::Float(v) => Value::Float(Some(*v)),
            CellValue::Double(v) => Value::Double(Some(*v)),
            CellValue::Text(Some(v)) => Value::String(Some(v.clone())),
            CellValue::Text(None) => Value::String(None),
            CellValue::Boolean(v) => Value::Bool(Some(*v)),
            CellValue::Date(v) => Value::ChronoDate(Some(*v)),
            CellValue::Time(v) => Value::ChronoTime(Some(*v)),
            CellValue::Timestamp(v) => Value::ChronoDateTime(Some(*v)),
        }
    }

    // 加载子数据集
    async fn load_child_datasets(
        &self,
        tx: &DatabaseTransaction,
        schema: &TableSchema,
        parent_table: &str,
        parent_id: &Value,
    ) -> Result<Option<HashMap<String, DataSet>>, DataSetError> {
        // 查询表关系定义
        let relations = TableRelation::find()
            .filter(Condition::all().add(
                sea_orm::ColumnTrait::eq(
                    sea_orm::ColumnDef::new("parent_table"),
                    parent_table,
                ),
            ))
            .all(tx)
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        if relations.is_empty() {
            return Ok(None);
        }

        let mut children = HashMap::new();

        for relation in relations {
            // 构建子表查询
            let mut query_builder = QueryBuilder::new(format!(
                "SELECT * FROM {} WHERE {} = :parent_id",
                relation.child_table, relation.child_key
            ));
            
            let mut params = HashMap::new();
            params.insert("parent_id".to_string(), parent_id.clone());

            // 加载子数据集
            let child_schema = schema.get_child_schema(&relation.child_table)?;
            let child_dataset = self.load_dataset(
                &child_schema,
                &query_builder.to_string(),
                &params,
            ).await?;

            children.insert(relation.relation_name, child_dataset);
        }

        Ok(Some(children))
    }

    // 保存子数据集
    async fn save_child_datasets(
        &self,
        tx: &DatabaseTransaction,
        schema: &TableSchema,
        row: &RowSet,
        parent_id: &Value,
    ) -> Result<(), DataSetError> {
        if let Some(children) = &row.children {
            for (relation_name, child_dataset) in children {
                // 获取关系定义
                let relation = TableRelation::find()
                    .filter(
                        Condition::all()
                            .add(sea_orm::ColumnTrait::eq(
                                sea_orm::ColumnDef::new("parent_table"),
                                &schema.table_name,
                            ))
                            .add(sea_orm::ColumnTrait::eq(
                                sea_orm::ColumnDef::new("relation_name"),
                                relation_name,
                            )),
                    )
                    .one(tx)
                    .await
                    .map_err(|e| DataSetError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| DataSetError::RelationNotFound(relation_name.clone()))?;

                // 保存子数据集
                let child_schema = schema.get_child_schema(&relation.child_table)?;
                self.save_dataset(
                    &child_schema,
                    child_dataset,
                    Some((&relation.child_key, parent_id)),
                ).await?;
            }
        }
        Ok(())
    }
}

#[async_trait]
impl SeaOrmDataSetStorage for SeaOrmStorage {
    async fn load_dataset(
        &self,
        schema: &TableSchema,
        query: &str,
        params: &HashMap<String, Value>,
    ) -> Result<DataSet, DataSetError> {
        let tx = self.db
            .begin()
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        let mut dataset = DataSet::new();

        // 执行查��
        let mut query_builder = QueryBuilder::new(query);
        for (name, value) in params {
            query_builder = query_builder.bind(value.clone());
        }

        let result = query_builder
            .build()
            .query(&tx)
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        // 处理查询结果
        while let Some(row) = result.try_next().await.map_err(|e| DataSetError::DatabaseError(e.to_string()))? {
            let mut rowset = RowSet::new(schema.column_count());

            // 设置基本列值
            for i in 0..schema.column_count() {
                let col_name = &schema.columns[i].name;
                let value = row.try_get::<Value>(col_name)
                    .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;
                rowset.set_value(i, Self::convert_to_cell_value(&value)?)?;
            }

            // 获取主键值
            let id_value = rowset.get_value(schema.get_primary_key_index()?)?;
            let id_sea_value = Self::convert_to_sea_value(id_value);

            // 加载子数据集
            if let Some(children) = self.load_child_datasets(
                &tx,
                schema,
                &schema.table_name,
                &id_sea_value,
            ).await? {
                for (name, child_dataset) in children {
                    rowset.add_child(name, child_dataset);
                }
            }

            dataset.add_row(schema, rowset.values)?;
        }

        tx.commit()
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        Ok(dataset)
    }

    async fn save_dataset(
        &self,
        schema: &TableSchema,
        dataset: &DataSet,
        parent_ref: Option<(&str, &Value)>,
    ) -> Result<(), DataSetError> {
        let tx = self.db
            .begin()
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        if let Some(rows) = &dataset.rows {
            for row in rows {
                // 构建插入语句
                let mut columns = schema.columns
                    .iter()
                    .map(|col| col.name.as_str())
                    .collect::<Vec<_>>();
                let mut values = Vec::new();

                // 添加父引用
                if let Some((parent_key, parent_value)) = parent_ref {
                    columns.push(parent_key);
                    values.push(parent_value.clone());
                }

                // 添加其他列值
                for i in 0..schema.column_count() {
                    let value = row.get_value(i)?;
                    values.push(Self::convert_to_sea_value(value));
                }

                // 构建并执行插入
                let mut query_builder = QueryBuilder::new(format!(
                    "INSERT INTO {} ({}) VALUES ({}) RETURNING id",
                    schema.table_name,
                    columns.join(", "),
                    (0..values.len()).map(|i| format!("${}", i + 1)).collect::<Vec<_>>().join(", ")
                ));

                for value in values {
                    query_builder = query_builder.bind(value);
                }

                let id: i64 = query_builder
                    .build()
                    .query_scalar(&tx)
                    .await
                    .map_err(|e| DataSetError::DatabaseError(e.to_string()))?
                    .ok_or_else(|| DataSetError::DatabaseError("Failed to get inserted ID".to_string()))?;

                // 保存子数据集
                self.save_child_datasets(
                    &tx,
                    schema,
                    row,
                    &Value::BigInt(Some(id)),
                ).await?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

// 测试代码
#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::Database;

    #[tokio::test]
    async fn test_seaorm_storage() -> Result<(), Box<dyn std::error::Error>> {
        // 连接数据库
        let db = Database::connect("postgres://username:password@localhost/dbname")
            .await?;
        let storage = SeaOrmStorage::new(db);

        // 创建测试 schema
        let schema = TableSchema::new(
            "departments".to_string(),
            vec![
                ColumnDef::new("id".into(), ColumnType::Integer),
                ColumnDef::new("name".into(), ColumnType::Text),
            ],
        );

        // 加载数据
        let mut params = HashMap::new();
        params.insert("company_id".to_string(), Value::Int(Some(1)));

        let dataset = storage.load_dataset(
            &schema,
            "SELECT * FROM departments WHERE company_id = :company_id",
            &params,
        ).await?;

        // 保存数据
        storage.save_dataset(&schema, &dataset, None).await?;

        Ok(())
    }
} 