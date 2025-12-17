use sqlx::{Pool, Row, Postgres, Error as SqlxError};
use async_trait::async_trait;
use super::{DataSet, TableSchema, CellValue, DataSetError};

#[async_trait]
pub trait DataSetStorage {
    async fn load_dataset(&self, schema: &TableSchema, query: &str) -> Result<DataSet, DataSetError>;
    async fn save_dataset(&self, schema: &TableSchema, dataset: &DataSet) -> Result<(), DataSetError>;
}

pub struct PostgresDataSetStorage {
    pool: Pool<Postgres>,
}

impl PostgresDataSetStorage {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    // 将数据库值转换为 CellValue
    fn convert_to_cell_value<'a>(row: &'a sqlx::postgres::PgRow, col_idx: usize) -> Result<CellValue, DataSetError> {
        let value = if let Ok(null) = row.try_get_raw(col_idx) {
            if null.is_null() {
                return Ok(CellValue::Null);
            }
        };

        // 根据实际类型进行转换
        if let Ok(val) = row.try_get::<i32, _>(col_idx) {
            Ok(CellValue::Integer(val))
        } else if let Ok(val) = row.try_get::<i64, _>(col_idx) {
            Ok(CellValue::BigInt(val))
        } else if let Ok(val) = row.try_get::<f32, _>(col_idx) {
            Ok(CellValue::Float(val))
        } else if let Ok(val) = row.try_get::<f64, _>(col_idx) {
            Ok(CellValue::Double(val))
        } else if let Ok(val) = row.try_get::<String, _>(col_idx) {
            Ok(CellValue::Text(Some(val)))
        } else if let Ok(val) = row.try_get::<bool, _>(col_idx) {
            Ok(CellValue::Boolean(val))
        } else if let Ok(val) = row.try_get::<chrono::NaiveDate, _>(col_idx) {
            Ok(CellValue::Date(val))
        } else if let Ok(val) = row.try_get::<chrono::NaiveTime, _>(col_idx) {
            Ok(CellValue::Time(val))
        } else if let Ok(val) = row.try_get::<chrono::NaiveDateTime, _>(col_idx) {
            Ok(CellValue::Timestamp(val))
        } else {
            Err(DataSetError::TypeConversionError)
        }
    }

    // 将 CellValue 转换为 SQL 参数值
    fn convert_to_sql_value(value: &CellValue) -> String {
        match value {
            CellValue::Null => "NULL".to_string(),
            CellValue::Integer(v) => v.to_string(),
            CellValue::BigInt(v) => v.to_string(),
            CellValue::Float(v) => v.to_string(),
            CellValue::Double(v) => v.to_string(),
            CellValue::Text(Some(v)) => format!("'{}'", v.replace('\'', "''")),
            CellValue::Text(None) => "NULL".to_string(),
            CellValue::Boolean(v) => v.to_string(),
            CellValue::Date(v) => format!("'{}'", v),
            CellValue::Time(v) => format!("'{}'", v),
            CellValue::Timestamp(v) => format!("'{}'", v),
        }
    }
}

#[async_trait]
impl DataSetStorage for PostgresDataSetStorage {
    async fn load_dataset(&self, schema: &TableSchema, query: &str) -> Result<DataSet, DataSetError> {
        let mut dataset = DataSet::new();

        // 执行查询
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        // 处理每一行数据
        for row in rows {
            let mut values = Vec::with_capacity(schema.column_count());
            
            // 处理每一列
            for i in 0..schema.column_count() {
                let value = Self::convert_to_cell_value(&row, i)?;
                values.push(value);
            }

            // 添加到数据集
            dataset.add_row(schema, values)?;
        }

        Ok(dataset)
    }

    async fn save_dataset(&self, schema: &TableSchema, dataset: &DataSet) -> Result<(), DataSetError> {
        // 开始事务
        let mut tx = self.pool
            .begin()
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        // 构建列名列表
        let columns = schema.columns
            .iter()
            .map(|col| col.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");

        // 处理每一行数据
        for i in 0..dataset.row_count() {
            let row = dataset.get_row(i)?;
            
            // 构建值列表
            let values = (0..schema.column_count())
                .map(|j| {
                    let value = row.get_value(j)
                        .map_err(|_| DataSetError::IndexOutOfBounds)?;
                    Ok(Self::convert_to_sql_value(value))
                })
                .collect::<Result<Vec<_>, DataSetError>>()?
                .join(", ");

            // 构建并执行插入语句
            let sql = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                schema.table_name, columns, values
            );

            sqlx::query(&sql)
                .execute(&mut tx)
                .await
                .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;
        }

        // 提交事务
        tx.commit()
            .await
            .map_err(|e| DataSetError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

// 在 DataSetError 中添加新的错误类型
#[derive(Error, Debug)]
pub enum DataSetError {
    // ... 现有错误类型 ...
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Type conversion error")]
    TypeConversionError,
}

// 使用示例
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn test_dataset_db_operations() -> Result<(), Box<dyn std::error::Error>> {
        // 创建数据库连接池
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://username:password@localhost/dbname")
            .await?;

        // 创建存储实例
        let storage = PostgresDataSetStorage::new(pool);

        // 创建表结构
        let schema = TableSchema::new(
            "employees".to_string(),
            vec![
                ColumnDef::new("id".into(), ColumnType::Integer),
                ColumnDef::new("name".into(), ColumnType::Text),
                ColumnDef::new("salary".into(), ColumnType::Double),
            ],
        );

        // 加载数据
        let query = "SELECT id, name, salary FROM employees WHERE department_id = 1";
        let dataset = storage.load_dataset(&schema, query).await?;

        // 处理数据...

        // 保存数据
        storage.save_dataset(&schema, &dataset).await?;

        Ok(())
    }
}

#[async_trait]
pub trait HierarchicalDataSetStorage: DataSetStorage {
    async fn load_hierarchical_dataset(
        &self,
        schema: &TableSchema,
        parent_query: &str,
        child_queries: &[(&str, String)],
    ) -> Result<DataSet, DataSetError> {
        // 加载主数据集
        let mut dataset = self.load_dataset(schema, parent_query).await?;

        // 加载每个子数据集
        for (i, row) in dataset.rows.as_ref().unwrap().iter_mut().enumerate() {
            for (child_id, query) in child_queries {
                // 替换查询中的参数（例如父记录ID）
                let query = query.replace(
                    ":parent_id",
                    &row.get_value(0)?.to_string(),
                );

                // 递归加载子数据集
                let child_dataset = self.load_dataset(schema, &query).await?;
                row.add_child(child_id.to_string(), child_dataset);
            }
        }

        Ok(dataset)
    }

    async fn save_hierarchical_dataset(
        &self,
        schema: &TableSchema,
        dataset: &DataSet,
    ) -> Result<(), DataSetError> {
        // 保存主数据集
        self.save_dataset(schema, dataset).await?;

        // 递归保存子数据集
        if let Some(rows) = &dataset.rows {
            for row in rows {
                if let Some(children) = &row.children {
                    for (_, child_dataset) in children {
                        self.save_hierarchical_dataset(schema, child_dataset).await?;
                    }
                }
            }
        }

        Ok(())
    }
}

// 为 PostgresDataSetStorage 实现层级存储特征
impl HierarchicalDataSetStorage for PostgresDataSetStorage {}

// 使用示例
#[tokio::test]
async fn test_hierarchical_dataset_db_operations() -> Result<(), Box<dyn std::error::Error>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://username:password@localhost/dbname")
        .await?;

    let storage = PostgresDataSetStorage::new(pool);

    // 定义查询
    let parent_query = "SELECT * FROM departments";
    let child_queries = vec![
        ("employees", "SELECT * FROM employees WHERE department_id = :parent_id".to_string()),
        ("projects", "SELECT * FROM projects WHERE department_id = :parent_id".to_string()),
    ];

    // 加载层级数据
    let schema = TableSchema::new(/* ... */);
    let dataset = storage.load_hierarchical_dataset(&schema, parent_query, &child_queries).await?;

    // 保存层级数据
    storage.save_hierarchical_dataset(&schema, &dataset).await?;

    Ok(())
}
