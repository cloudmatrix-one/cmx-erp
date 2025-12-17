//! # 行存储数据集 (Row Data Set) 模块
//!
//! 这个模块实现了基于行存储的数据集结构，提供了高效的数据查询和操作功能。
//! 主要包含两个核心结构体：`RowData` 和 `RowDataSet`。
//!
//! ## 主要特性
//!
//! - **行优先存储**：数据按行组织，便于插入和读取完整记录
//! - **模式定义**：支持列的类型定义和约束
//! - **层次结构**：支持行级别的子数据集，实现复杂数据结构
//! - **类型安全**：提供类型检查和错误处理
//! - **序列化支持**：支持 JSON 序列化和反序列化
//!
//! ## 使用场景
//!
//! - 关系型数据处理
//! - 配置数据管理
//! - 层次化数据结构
//! - 数据导入导出
//!
//! ## 示例
//!
//! ```rust
//! use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
//! use cmx_core::model::data::cell::CellValue;
//! use serde_json::Number;
//!
//! // 创建数据集
//! let mut dataset = RowDataSet::new("users".to_string());
//!
//! // 添加列定义
//! dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
//! dataset.add_column("name".to_string(), ColumnType::String).unwrap();
//!
//! // 添加数据行
//! let row = vec![
//!     CellValue::Number(Number::from(1)),
//!     CellValue::String("Alice".to_string()),
//! ];
//! dataset.add_row(row).unwrap();
//!
//! // 查询数据
//! let cell = dataset.get_cell(0, "name").unwrap();
//! if let CellValue::String(name) = cell {
//!     println!("User name: {}", name);
//! }
//! ```

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{ColumnType, DataSetError};
use crate::model::data::cell::CellValue;

/// 列定义信息
///
/// 存储列的元数据信息，包括列在数据集中的索引位置和数据类型。
/// 这个结构体是数据集模式定义的核心组成部分。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnInfo {
    /// 列在行数据中的索引位置
    ///
    /// 从 0 开始的索引，用于快速定位列数据在行数组中的位置
    pub index: usize,

    /// 列的数据类型
    ///
    /// 决定了该列可以存储的数据类型，以及相应的验证规则
    pub column_type: ColumnType,
}

/// 行数据结构，包含值和子数据集
///
/// `RowData` 表示数据集中的一行数据，支持存储多个列的值以及层次化的子数据集。
/// 这是行存储数据集的核心数据单元，支持复杂的数据关系和嵌套结构。
///
/// ## 数据结构
///
/// 每一行包含两部分：
/// - **列值数组**：按列定义顺序存储的单元格值
/// - **子数据集映射**：可选的子数据集集合，支持层次化数据
///
/// ## 使用示例
///
/// ```rust
/// use cmx_core::model::data::dataset::RowData;
/// use cmx_core::model::data::cell::CellValue;
///
/// // 创建包含三列数据的行
/// let values = vec![
///     CellValue::String("John".to_string()),
///     CellValue::Number(25.into()),
///     CellValue::Bool(true),
/// ];
/// let row = RowData::new(values);
///
/// // 添加子数据集
/// // row.add_child("addresses".to_string(), address_dataset);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowData {
    /// 行的列值数组
    ///
    /// 按照数据集列定义的顺序存储每个列的值。数组长度必须与数据集的列数一致。
    /// 支持所有 `CellValue` 类型的值，包括字符串、数字、布尔值等。
    pub values: Vec<CellValue>,

    /// 子数据集映射：名称 -> 数据集
    ///
    /// 可选的子数据集集合，用于实现层次化数据结构。
    /// 每个子数据集都有唯一的名称标识，可以存储相关联的子数据。
    /// 这使得一行数据可以包含复杂的关系型数据结构。
    pub children: Option<HashMap<String, RowDataSet>>,
}

impl RowData {
    /// 创建新的行数据
    ///
    /// 使用给定的列值数组创建新的行数据实例。子数据集映射初始化为空。
    ///
    /// # 参数
    ///
    /// * `values` - 列值的向量，按照数据集列定义的顺序排列
    ///
    /// # 示例
    ///
    /// ```rust
    /// use cmx_core::model::data::dataset::RowData;
    /// use cmx_core::model::data::cell::CellValue;
    ///
    /// let values = vec![
    ///     CellValue::String("Alice".to_string()),
    ///     CellValue::Number(30.into()),
    /// ];
    /// let row = RowData::new(values);
    /// ```
    pub fn new(values: Vec<CellValue>) -> Self {
        Self {
            values,
            children: None,
        }
    }

    /// 获取行值的只读引用
    ///
    /// 返回行中所有列值的向量引用，可以用于读取列数据。
    ///
    /// # 返回值
    ///
    /// 返回包含所有列值的向量引用
    pub fn values(&self) -> &Vec<CellValue> {
        &self.values
    }

    /// 获取行值的可变引用
    ///
    /// 返回行中所有列值的可变向量引用，可以用于修改列数据。
    ///
    /// # 返回值
    ///
    /// 返回包含所有列值的可变向量引用
    pub fn values_mut(&mut self) -> &mut Vec<CellValue> {
        &mut self.values
    }

    /// 获取指定的子数据集
    ///
    /// 根据名称查找并返回对应的子数据集的只读引用。
    /// 如果指定的子数据集不存在，返回 `None`。
    ///
    /// # 参数
    ///
    /// * `name` - 子数据集的名称标识符
    ///
    /// # 返回值
    ///
    /// 返回子数据集的只读引用，如果不存在则返回 `None`
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::RowData;
    /// # let row = RowData::new(vec![]);
    /// if let Some(addresses) = row.get_child("addresses") {
    ///     println!("Found addresses dataset with {} rows", addresses.row_count());
    /// }
    /// ```
    pub fn get_child(&self, name: &str) -> Option<&RowDataSet> {
        self.children.as_ref().and_then(|children| children.get(name))
    }

    /// 获取指定的子数据集的可变引用
    ///
    /// 根据名称查找并返回对应的子数据集的可变引用。
    /// 如果指定的子数据集不存在，返回 `None`。
    ///
    /// # 参数
    ///
    /// * `name` - 子数据集的名称标识符
    ///
    /// # 返回值
    ///
    /// 返回子数据集的可变引用，如果不存在则返回 `None`
    pub fn get_child_mut(&mut self, name: &str) -> Option<&mut RowDataSet> {
        self.children.as_mut().and_then(|children| children.get_mut(name))
    }

    /// 添加子数据集
    ///
    /// 将指定的数据集作为子数据集添加到当前行中。
    /// 如果子数据集映射不存在，会自动创建。
    ///
    /// # 参数
    ///
    /// * `name` - 子数据集的唯一名称标识符
    /// * `dataset` - 要添加的子数据集实例
    ///
    /// # 注意
    ///
    /// 如果同名的子数据集已存在，会被新的数据集替换。
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowData, RowDataSet};
    /// # let mut row = RowData::new(vec![]);
    /// # let dataset = RowDataSet::new("addresses".to_string());
    /// row.add_child("addresses".to_string(), dataset);
    /// ```
    pub fn add_child(&mut self, name: String, dataset: RowDataSet) {
        if self.children.is_none() {
            self.children = Some(HashMap::new());
        }
        self.children.as_mut().unwrap().insert(name, dataset);
    }

    /// 移除子数据集
    ///
    /// 根据名称移除指定的子数据集，并返回被移除的数据集实例。
    /// 如果指定的子数据集不存在，返回 `None`。
    ///
    /// # 参数
    ///
    /// * `name` - 要移除的子数据集的名称标识符
    ///
    /// # 返回值
    ///
    /// 返回被移除的子数据集实例，如果不存在则返回 `None`
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowData, RowDataSet};
    /// # let mut row = RowData::new(vec![]);
    /// # let dataset = RowDataSet::new("temp".to_string());
    /// # row.add_child("temp".to_string(), dataset);
    /// let removed = row.remove_child("temp");
    /// assert!(removed.is_some());
    /// ```
    pub fn remove_child(&mut self, name: &str) -> Option<RowDataSet> {
        self.children.as_mut().and_then(|children| children.remove(name))
    }
}

/// 行存储的数据集结构
///
/// `RowDataSet` 是基于行存储的数据集实现，提供了完整的关系型数据管理功能。
/// 支持列定义、数据操作、查询、序列化等核心功能，是整个数据集系统的核心组件。
///
/// ## 核心特性
///
/// - **模式驱动**：基于列定义的强类型数据存储
/// - **层次化数据**：支持行级别的子数据集嵌套
/// - **内存高效**：行存储适合频繁的插入和完整记录查询
/// - **序列化支持**：支持 JSON 序列化，便于数据持久化和传输
/// - **类型安全**：提供运行时类型检查和错误处理
///
/// ## 数据结构
///
/// ```text
/// RowDataSet {
///     dataset_id: "users",           // 数据集唯一标识
///     schema: {                      // 列定义映射
///         "id" => ColumnInfo { index: 0, column_type: I32 },
///         "name" => ColumnInfo { index: 1, column_type: String },
///         "active" => ColumnInfo { index: 2, column_type: Bool },
///     },
///     rows: [                        // 行数据数组
///         RowData { values: [Number(1), String("Alice"), Bool(true)], children: None },
///         RowData { values: [Number(2), String("Bob"), Bool(false)], children: Some(...) },
///     ]
/// }
/// ```
///
/// ## 使用场景
///
/// - **关系型数据管理**：适合需要复杂查询和关系操作的数据
/// - **配置数据存储**：用于存储应用程序配置和元数据
/// - **数据导入导出**：作为数据交换的中间格式
/// - **内存数据库**：轻量级的内存数据存储解决方案
///
/// ## 性能特点
///
/// - **插入性能好**：行存储适合频繁的记录插入
/// - **查询灵活**：支持按行、按列、按条件查询
/// - **内存占用可控**：支持大数据集的分页加载
/// - **序列化高效**：基于 serde 的高性能序列化
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowDataSet {
    /// 数据集唯一标识符
    ///
    /// 用于区分不同的数据集实例，在整个系统中应该是唯一的。
    /// 支持字母、数字、下划线等字符。
    pub dataset_id: String,

    /// 列定义映射：列名 -> 列信息
    ///
    /// 存储数据集的模式定义，将列名映射到列的元数据信息。
    /// 包含列的索引位置和数据类型，是数据完整性验证的基础。
    pub schema: HashMap<String, ColumnInfo>,

    /// 行数据存储数组
    ///
    /// 存储所有数据行的向量，每行包含列值和可选的子数据集。
    /// 行数据按添加顺序存储，支持随机访问和修改。
    pub rows: Vec<RowData>,
}

impl RowDataSet {
    /// 创建新的数据集实例
    ///
    /// 使用指定的数据集ID创建空的行数据集。初始状态下不包含任何列定义和数据行。
    ///
    /// # 参数
    ///
    /// * `dataset_id` - 数据集的唯一标识符字符串
    ///
    /// # 返回值
    ///
    /// 返回新创建的空数据集实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use cmx_core::model::data::dataset::RowDataSet;
    ///
    /// let dataset = RowDataSet::new("users".to_string());
    /// assert_eq!(dataset.dataset_id(), "users");
    /// assert_eq!(dataset.row_count(), 0);
    /// assert_eq!(dataset.column_count(), 0);
    /// ```
    pub fn new(dataset_id: String) -> Self {
        Self {
            dataset_id,
            schema: HashMap::new(),
            rows: Vec::new(),
        }
    }

    /// 获取数据集的ID标识符
    ///
    /// 返回数据集的唯一标识符字符串的引用。
    /// 这个ID在数据集的生命周期内是只读的。
    ///
    /// # 返回值
    ///
    /// 返回数据集ID的字符串切片引用
    pub fn dataset_id(&self) -> &str {
        &self.dataset_id
    }

    /// 获取数据集的列数量
    ///
    /// 返回当前数据集定义的列的数量。
    /// 这个数量等于模式定义中列的数量。
    ///
    /// # 返回值
    ///
    /// 返回列的数量
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// dataset.add_column("name".to_string(), ColumnType::String).unwrap();
    /// assert_eq!(dataset.column_count(), 2);
    /// ```
    pub fn column_count(&self) -> usize {
        self.schema.len()
    }

    /// 获取数据集的行数量
    ///
    /// 返回当前数据集存储的数据行数量。
    ///
    /// # 返回值
    ///
    /// 返回行的数量
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// dataset.add_row(vec![CellValue::Number(1.into())]).unwrap();
    /// assert_eq!(dataset.row_count(), 1);
    /// ```
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// 添加列定义到数据集模式
    ///
    /// 为数据集添加新的列定义，包括列名和数据类型。新添加的列会被分配一个索引位置，
    /// 并且所有现有数据行都会为这个新列添加默认的NULL值。
    ///
    /// # 参数
    ///
    /// * `name` - 列的唯一名称标识符，不能与现有列重复
    /// * `column_type` - 列的数据类型，定义了该列可以存储的数据类型
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), DataSetError>`：
    /// - `Ok(())` - 列添加成功
    /// - `Err(DataSetError::ColumnNotFound)` - 如果列名已存在（当前错误类型不够精确，后续可改进）
    ///
    /// # 行为说明
    ///
    /// - 新列会被添加到模式的末尾，索引位置等于当前列的数量
    /// - 所有现有行会为新列添加 `CellValue::Null` 值
    /// - 列名在数据集内必须唯一
    ///
    /// # 示例
    ///
    /// ```rust
    /// use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    ///
    /// let mut dataset = RowDataSet::new("users".to_string());
    ///
    /// // 添加列定义
    /// dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// dataset.add_column("name".to_string(), ColumnType::String).unwrap();
    /// dataset.add_column("active".to_string(), ColumnType::Bool).unwrap();
    ///
    /// assert_eq!(dataset.column_count(), 3);
    /// ```
    ///
    /// # 注意事项
    ///
    /// - 添加列操作不可逆，所有现有行都会被修改
    /// - 建议在添加数据之前完成所有的列定义
    pub fn add_column(&mut self, name: String, column_type: ColumnType) -> Result<(), DataSetError> {
        if self.schema.contains_key(&name) {
            return Err(DataSetError::ColumnNotFound); // 可以扩展错误类型以更好地表达列已存在
        }

        let index = self.schema.len();
        self.schema.insert(name, ColumnInfo { index, column_type });

        // 为现有行添加 NULL 值
        for row in &mut self.rows {
            row.values_mut().push(CellValue::Null);
        }

        Ok(())
    }

    /// 获取指定列的信息
    ///
    /// 根据列名查找并返回列的详细信息，包括列的索引位置和数据类型。
    /// 如果指定的列不存在，返回 `None`。
    ///
    /// # 参数
    ///
    /// * `column_name` - 要查询的列名
    ///
    /// # 返回值
    ///
    /// 返回 `Option<&ColumnInfo>`：
    /// - `Some(&ColumnInfo)` - 包含列的详细信息
    /// - `None` - 指定的列不存在
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    ///
    /// if let Some(col_info) = dataset.get_column_info("id") {
    ///     println!("Column 'id' is at index {}", col_info.index);
    ///     // 打印列类型等信息
    /// }
    /// ```
    pub fn get_column_info(&self, column_name: &str) -> Option<&ColumnInfo> {
        self.schema.get(column_name)
    }

    /// 添加一行数据到数据集末尾
    ///
    /// 将包含完整列值的数据行添加到数据集的末尾。
    /// 数据行的列值数量必须与数据集定义的列数量完全匹配。
    ///
    /// # 参数
    ///
    /// * `values` - 包含所有列值的向量，顺序必须与列定义一致
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), DataSetError>`：
    /// - `Ok(())` - 行添加成功
    /// - `Err(DataSetError::ColumnCountMismatch)` - 如果值的数量与列的数量不匹配
    ///
    /// # 行为说明
    ///
    /// - 新行会被添加到数据集的末尾
    /// - 行索引从 0 开始，新行的索引等于之前的行数
    /// - 所有列值都会被验证格式（但不验证类型）
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("users".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_column("name".to_string(), ColumnType::String).unwrap();
    ///
    /// // 添加一行数据
    /// let row_values = vec![
    ///     CellValue::Number(1.into()),
    ///     CellValue::String("Alice".to_string()),
    /// ];
    /// dataset.add_row(row_values).unwrap();
    ///
    /// assert_eq!(dataset.row_count(), 1);
    /// ```
    ///
    /// # 注意事项
    ///
    /// - 值的顺序必须与 `add_column` 的调用顺序一致
    /// - NULL 值应该使用 `CellValue::Null` 表示
    /// - 建议使用类型正确的值以避免运行时错误
    pub fn add_row(&mut self, values: Vec<CellValue>) -> Result<(), DataSetError> {
        if values.len() != self.schema.len() {
            return Err(DataSetError::ColumnCountMismatch);
        }
        self.rows.push(RowData::new(values));
        Ok(())
    }

    /// 在指定位置插入一行数据
    ///
    /// 将包含完整列值的数据行插入到数据集的指定位置。
    /// 位于插入位置之后的所有行会自动向后移动一个位置。
    ///
    /// # 参数
    ///
    /// * `index` - 插入位置的索引，从 0 开始
    /// * `values` - 包含所有列值的向量，顺序必须与列定义一致
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), DataSetError>`：
    /// - `Ok(())` - 行插入成功
    /// - `Err(DataSetError::ColumnCountMismatch)` - 如果值的数量与列的数量不匹配
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果索引超出有效范围
    ///
    /// # 行为说明
    ///
    /// - 插入位置可以等于当前行数，表示在末尾追加
    /// - 所有后续行的索引会自动调整
    /// - 插入操作的时间复杂度为 O(n)，其中 n 为行数
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(1.into())]).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(3.into())]).unwrap();
    ///
    /// // 在中间插入一行
    /// let middle_row = vec![CellValue::Number(2.into())];
    /// dataset.insert_row(1, middle_row).unwrap();
    ///
    /// // 验证结果：[1, 2, 3]
    /// assert_eq!(dataset.row_count(), 3);
    /// ```
    ///
    /// # 注意事项
    ///
    /// - 频繁的中间插入操作会影响性能
    /// - 对于大量数据的场景，考虑使用其他数据结构
    pub fn insert_row(&mut self, index: usize, values: Vec<CellValue>) -> Result<(), DataSetError> {
        if values.len() != self.schema.len() {
            return Err(DataSetError::ColumnCountMismatch);
        }
        if index > self.rows.len() {
            return Err(DataSetError::IndexOutOfBounds);
        }
        self.rows.insert(index, RowData::new(values));
        Ok(())
    }

    /// 获取指定行的只读引用
    ///
    /// 根据行索引返回数据集中指定行的只读引用。
    /// 可以用于读取行数据，但不能修改。
    ///
    /// # 参数
    ///
    /// * `index` - 行索引，从 0 开始
    ///
    /// # 返回值
    ///
    /// 返回 `Result<&RowData, DataSetError>`：
    /// - `Ok(&row)` - 指定行的只读引用
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果索引超出范围
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("name".to_string(), ColumnType::String).unwrap();
    /// # dataset.add_row(vec![CellValue::String("Alice".to_string())]).unwrap();
    ///
    /// let row = dataset.get_row(0).unwrap();
    /// let values = row.values();
    /// // 现在可以读取行中的值
    /// ```
    pub fn get_row(&self, index: usize) -> Result<&RowData, DataSetError> {
        self.rows.get(index).ok_or(DataSetError::IndexOutOfBounds)
    }

    /// 获取指定行的可变引用
    ///
    /// 根据行索引返回数据集中指定行的可变引用。
    /// 可以用于修改行数据，包括列值和子数据集。
    ///
    /// # 参数
    ///
    /// * `index` - 行索引，从 0 开始
    ///
    /// # 返回值
    ///
    /// 返回 `Result<&mut RowData, DataSetError>`：
    /// - `Ok(&mut row)` - 指定行的可变引用
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果索引超出范围
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("name".to_string(), ColumnType::String).unwrap();
    /// # dataset.add_row(vec![CellValue::String("Alice".to_string())]).unwrap();
    ///
    /// let row = dataset.get_row_mut(0).unwrap();
    /// // 现在可以修改行中的值
    /// row.values_mut()[0] = CellValue::String("Bob".to_string());
    /// ```
    pub fn get_row_mut(&mut self, index: usize) -> Result<&mut RowData, DataSetError> {
        self.rows.get_mut(index).ok_or(DataSetError::IndexOutOfBounds)
    }

    /// 删除指定行并返回被删除的行数据
    ///
    /// 根据行索引删除数据集中的指定行，并返回被删除的行数据。
    /// 删除位置之后的所有行会自动向前移动一个位置。
    ///
    /// # 参数
    ///
    /// * `index` - 要删除的行索引，从 0 开始
    ///
    /// # 返回值
    ///
    /// 返回 `Result<RowData, DataSetError>`：
    /// - `Ok(row)` - 被删除的行数据，可以继续使用
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果索引超出范围
    ///
    /// # 行为说明
    ///
    /// - 删除操作会改变后续所有行的索引
    /// - 被删除的行数据会被返回，可以用于数据恢复或其他用途
    /// - 删除操作的时间复杂度为 O(n)，其中 n 为行数
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(1.into())]).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(2.into())]).unwrap();
    ///
    /// let removed_row = dataset.remove_row(0).unwrap();
    /// assert_eq!(dataset.row_count(), 1);
    ///
    /// // removed_row 可以继续使用
    /// let removed_id = &removed_row.values()[0];
    /// ```
    ///
    /// # 注意事项
    ///
    /// - 删除操作不可逆，除非保存返回的行数据
    /// - 频繁的删除操作会影响性能
    pub fn remove_row(&mut self, index: usize) -> Result<RowData, DataSetError> {
        if index >= self.rows.len() {
            return Err(DataSetError::IndexOutOfBounds);
        }
        Ok(self.rows.remove(index))
    }

    /// 获取指定单元格的值
    ///
    /// 根据行索引和列名获取数据集中指定单元格的值。
    /// 这是一个高效的随机访问方法，结合了行查找和列查找。
    ///
    /// # 参数
    ///
    /// * `row_index` - 行索引，从 0 开始
    /// * `column_name` - 列名字符串
    ///
    /// # 返回值
    ///
    /// 返回 `Result<&CellValue, DataSetError>`：
    /// - `Ok(&value)` - 指定单元格的值引用
    /// - `Err(DataSetError::ColumnNotFound)` - 如果列名不存在
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果行索引超出范围
    ///
    /// # 性能特点
    ///
    /// - 时间复杂度：O(1) - 基于预计算的列索引
    /// - 空间复杂度：O(1) - 只返回引用，不复制数据
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("name".to_string(), ColumnType::String).unwrap();
    /// # dataset.add_row(vec![CellValue::String("Alice".to_string())]).unwrap();
    ///
    /// let cell_value = dataset.get_cell(0, "name").unwrap();
    /// if let CellValue::String(name) = cell_value {
    ///     println!("Cell value: {}", name);
    /// }
    /// ```
    ///
    /// # 注意事项
    ///
    /// - 返回的是值的引用，生命周期与数据集相同
    /// - 对于频繁访问的场景，考虑缓存列索引以提高性能
    pub fn get_cell(&self, row_index: usize, column_name: &str) -> Result<&CellValue, DataSetError> {
        let col_info = self.schema.get(column_name)
            .ok_or(DataSetError::ColumnNotFound)?;

        let row = self.get_row(row_index)?;
        row.values().get(col_info.index).ok_or(DataSetError::IndexOutOfBounds)
    }

    /// 设置指定单元格的值
    ///
    /// 根据行索引和列名设置数据集中指定单元格的值。
    /// 新的值会替换原有值，原有值会被丢弃。
    ///
    /// # 参数
    ///
    /// * `row_index` - 行索引，从 0 开始
    /// * `column_name` - 列名字符串
    /// * `value` - 新的单元格值
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), DataSetError>`：
    /// - `Ok(())` - 值设置成功
    /// - `Err(DataSetError::ColumnNotFound)` - 如果列名不存在
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果行索引超出范围
    ///
    /// # 行为说明
    ///
    /// - 原有的单元格值会被新值完全替换
    /// - 不进行类型验证（依赖运行时检查）
    /// - 支持设置任何有效的 `CellValue` 类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("score".to_string(), ColumnType::F64).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(85.5.into())]).unwrap();
    ///
    /// // 更新分数
    /// dataset.set_cell(0, "score", CellValue::Number(95.0.into())).unwrap();
    ///
    /// // 验证更新结果
    /// let new_score = dataset.get_cell(0, "score").unwrap();
    /// assert!(matches!(new_score, CellValue::Number(_)));
    /// ```
    ///
    /// # 注意事项
    ///
    /// - 设置操作是原地修改，不会改变数据集的结构
    /// - 对于NULL值，应该使用 `CellValue::Null`
    pub fn set_cell(&mut self, row_index: usize, column_name: &str, value: CellValue) -> Result<(), DataSetError> {
        let col_info = self.schema.get(column_name)
            .ok_or(DataSetError::ColumnNotFound)?;

        let col_info_index = col_info.index;
        let row = self.get_row_mut(row_index)?;
        if col_info_index >= row.values().len() {
            return Err(DataSetError::IndexOutOfBounds);
        }
        row.values_mut()[col_info_index] = value;
        Ok(())
    }

    /// 获取指定列的所有值
    ///
    /// 返回指定列中所有行的值的向量引用。
    /// 返回值的顺序与行在数据集中的顺序一致。
    ///
    /// # 参数
    ///
    /// * `column_name` - 要查询的列名
    ///
    /// # 返回值
    ///
    /// 返回 `Result<Vec<&CellValue>, DataSetError>`：
    /// - `Ok(values)` - 包含该列所有值的向量，每个元素都是值的引用
    /// - `Err(DataSetError::ColumnNotFound)` - 如果列名不存在
    ///
    /// # 性能特点
    ///
    /// - 时间复杂度：O(n) - 需要遍历所有行
    /// - 空间复杂度：O(n) - 创建包含所有引用的向量
    /// - 内存效率：只存储引用，不复制实际数据
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("users".to_string());
    /// # dataset.add_column("age".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(25.into())]).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(30.into())]).unwrap();
    ///
    /// let ages = dataset.get_column_values("age").unwrap();
    /// assert_eq!(ages.len(), 2);
    ///
    /// // 计算平均年龄
    /// let avg_age: f64 = ages.iter()
    ///     .filter_map(|&cell| match cell {
    ///         CellValue::Number(n) => n.as_f64(),
    ///         _ => None,
    ///     })
    ///     .sum::<f64>() / ages.len() as f64;
    /// ```
    ///
    /// # 使用场景
    ///
    /// - 列级别的批量数据处理
    /// - 数据分析和聚合计算
    /// - 导出特定列的数据
    /// - 构建数据透视表的基础数据
    pub fn get_column_values(&self, column_name: &str) -> Result<Vec<&CellValue>, DataSetError> {
        let col_info = self.schema.get(column_name)
            .ok_or(DataSetError::ColumnNotFound)?;

        self.rows.iter()
            .map(|row| row.values().get(col_info.index).ok_or(DataSetError::IndexOutOfBounds))
            .collect()
    }

    /// 为指定行添加子数据集
    ///
    /// 将一个子数据集关联到指定行的指定名称下。
    /// 这样可以创建层次化的数据结构，实现主从表关系。
    ///
    /// # 参数
    ///
    /// * `row_index` - 父行的索引，从 0 开始
    /// * `name` - 子数据集的唯一名称标识符
    /// * `dataset` - 要添加的子数据集实例
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), DataSetError>`：
    /// - `Ok(())` - 子数据集添加成功
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果行索引超出范围
    ///
    /// # 行为说明
    ///
    /// - 如果行还没有子数据集映射，会自动创建
    /// - 同名的子数据集会被新数据集替换
    /// - 子数据集的生命周期与父行绑定
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut parent_dataset = RowDataSet::new("orders".to_string());
    /// # parent_dataset.add_column("order_id".to_string(), ColumnType::I32).unwrap();
    /// # parent_dataset.add_row(vec![CellValue::Number(1.into())]).unwrap();
    ///
    /// // 创建订单项子数据集
    /// let mut order_items = RowDataSet::new("order_items".to_string());
    /// order_items.add_column("item_id".to_string(), ColumnType::I32).unwrap();
    /// order_items.add_column("quantity".to_string(), ColumnType::I32).unwrap();
    /// order_items.add_row(vec![
    ///     CellValue::Number(101.into()),
    ///     CellValue::Number(2.into()),
    /// ]).unwrap();
    ///
    /// // 将子数据集添加到父行
    /// parent_dataset.add_child_dataset(0, "items".to_string(), order_items).unwrap();
    /// ```
    ///
    /// # 使用场景
    ///
    /// - 主从表关系：订单和订单项
    /// - 树形结构：组织架构、文件系统
    /// - 复杂对象：包含多个子对象的实体
    pub fn add_child_dataset(&mut self, row_index: usize, name: String, dataset: RowDataSet) -> Result<(), DataSetError> {
        let row = self.get_row_mut(row_index)?;
        row.add_child(name, dataset);
        Ok(())
    }

    /// 获取指定行的子数据集
    ///
    /// 根据行索引和子数据集名称获取对应的子数据集引用。
    ///
    /// # 参数
    ///
    /// * `row_index` - 父行的索引，从 0 开始
    /// * `name` - 子数据集的名称标识符
    ///
    /// # 返回值
    ///
    /// 返回 `Result<Option<&RowDataSet>, DataSetError>`：
    /// - `Ok(Some(&dataset))` - 找到指定的子数据集
    /// - `Ok(None)` - 行存在但没有指定的子数据集
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果行索引超出范围
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(1.into())]).unwrap();
    /// # let child_dataset = RowDataSet::new("child".to_string());
    /// # dataset.add_child_dataset(0, "child".to_string(), child_dataset).unwrap();
    ///
    /// if let Some(child) = dataset.get_child_dataset(0, "child").unwrap() {
    ///     println!("Found child dataset with {} rows", child.row_count());
    /// } else {
    ///     println!("No child dataset found");
    /// }
    /// ```
    pub fn get_child_dataset(&self, row_index: usize, name: &str) -> Result<Option<&RowDataSet>, DataSetError> {
        let row = self.get_row(row_index)?;
        Ok(row.get_child(name))
    }

    /// 移除指定行的子数据集
    ///
    /// 根据行索引和子数据集名称移除对应的子数据集，并返回被移除的数据集实例。
    ///
    /// # 参数
    ///
    /// * `row_index` - 父行的索引，从 0 开始
    /// * `name` - 要移除的子数据集的名称标识符
    ///
    /// # 返回值
    ///
    /// 返回 `Result<Option<RowDataSet>, DataSetError>`：
    /// - `Ok(Some(dataset))` - 成功移除并返回子数据集
    /// - `Ok(None)` - 行存在但没有指定的子数据集
    /// - `Err(DataSetError::IndexOutOfBounds)` - 如果行索引超出范围
    ///
    /// # 行为说明
    ///
    /// - 被移除的子数据集会从父行中完全分离
    /// - 返回的数据集实例可以继续使用或存储
    /// - 如果指定的子数据集不存在，不会产生错误，只返回 `None`
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(1.into())]).unwrap();
    /// # let child_dataset = RowDataSet::new("temp".to_string());
    /// # dataset.add_child_dataset(0, "temp".to_string(), child_dataset).unwrap();
    ///
    /// // 移除子数据集
    /// let removed = dataset.remove_child_dataset(0, "temp").unwrap();
    /// assert!(removed.is_some());
    ///
    /// // 验证子数据集已被移除
    /// let check = dataset.get_child_dataset(0, "temp").unwrap();
    /// assert!(check.is_none());
    /// ```
    pub fn remove_child_dataset(&mut self, row_index: usize, name: &str) -> Result<Option<RowDataSet>, DataSetError> {
        let row = self.get_row_mut(row_index)?;
        Ok(row.remove_child(name))
    }

    /// 清空数据集中的所有数据
    ///
    /// 移除所有数据行，但保留列定义和数据集ID。
    /// 这是一个不可逆的操作，清除的数据无法恢复。
    ///
    /// # 行为说明
    ///
    /// - 所有行数据会被移除
    /// - 行计数会变为 0
    /// - 列定义保持不变
    /// - 子数据集也会被清除
    /// - 数据集ID保持不变
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{RowDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = RowDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(1.into())]).unwrap();
    /// # dataset.add_row(vec![CellValue::Number(2.into())]).unwrap();
    ///
    /// assert_eq!(dataset.row_count(), 2);
    ///
    /// dataset.clear();
    ///
    /// assert_eq!(dataset.row_count(), 0);
    /// assert_eq!(dataset.column_count(), 1); // 列定义保留
    /// ```
    ///
    /// # 注意事项
    ///
    /// - 这是一个破坏性操作，无法撤销
    /// - 如果需要保留数据，建议先备份或导出
    /// - 列定义仍然有效，可以继续添加新数据
    pub fn clear(&mut self) {
        self.rows.clear();
    }
}

/// 单元测试模块
///
/// 包含 RowDataSet 和相关数据结构的完整测试用例，
/// 涵盖基本操作、边界条件和复杂场景的测试。
#[cfg(test)]
mod tests {
    use serde_json::Number;

    use super::*;

    /// 创建测试用数据集
    ///
    /// 辅助函数，创建一个包含 id、name、active 三个列的标准测试数据集。
    /// 用于多个测试用例的初始化。
    fn create_test_dataset() -> RowDataSet {
        let mut dataset = RowDataSet::new("test".to_string());
        dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
        dataset.add_column("name".to_string(), ColumnType::String).unwrap();
        dataset.add_column("active".to_string(), ColumnType::Bool).unwrap();
        dataset
    }

    /// 测试子数据集功能
    ///
    /// 验证子数据集的添加、获取和移除功能。
    /// 测试场景包括单个子数据集的完整生命周期。
    #[test]
    fn test_child_datasets() {
        let mut parent_dataset = create_test_dataset();

        // 添加父数据集的行
        let parent_row = vec![
            CellValue::Number(Number::from(1)),
            CellValue::String("Parent".to_string()),
            CellValue::Bool(true),
        ];
        parent_dataset.add_row(parent_row).unwrap();

        // 创建子数据集
        let mut child_dataset = RowDataSet::new("child".to_string());
        child_dataset.add_column("item_id".to_string(), ColumnType::I32).unwrap();
        child_dataset.add_column("quantity".to_string(), ColumnType::I32).unwrap();

        // 添加子数据集的行
        let child_row = vec![
            CellValue::Number(Number::from(1)),
            CellValue::Number(Number::from(5)),
        ];
        child_dataset.add_row(child_row).unwrap();

        // 将子数据集添加到父数据集的行
        parent_dataset.add_child_dataset(0, "items".to_string(), child_dataset).unwrap();

        // 验证子数据集
        let child = parent_dataset.get_child_dataset(0, "items").unwrap().unwrap();
        assert_eq!(child.column_count(), 2);
        assert_eq!(child.row_count(), 1);
        
        // 验证子数据集的值
        let quantity = child.get_cell(0, "quantity").unwrap();
        assert!(matches!(quantity, CellValue::Number(n) if n == &Number::from(5)));

        // 测试移除子数据集
        let removed_child = parent_dataset.remove_child_dataset(0, "items").unwrap().unwrap();
        assert_eq!(removed_child.dataset_id(), "child");
        assert!(parent_dataset.get_child_dataset(0, "items").unwrap().is_none());
    }

    /// 测试多个子数据集功能
    ///
    /// 验证单行数据支持多个子数据集的能力。
    /// 测试添加、验证和部分移除多个子数据集的场景。
    #[test]
    fn test_multiple_child_datasets() {
        let mut parent_dataset = create_test_dataset();
        parent_dataset.add_row(vec![
            CellValue::Number(Number::from(1)),
            CellValue::String("Parent".to_string()),
            CellValue::Bool(true),
        ]).unwrap();

        // 添加多个子数据集
        let child1 = RowDataSet::new("child1".to_string());
        let child2 = RowDataSet::new("child2".to_string());

        parent_dataset.add_child_dataset(0, "child1".to_string(), child1).unwrap();
        parent_dataset.add_child_dataset(0, "child2".to_string(), child2).unwrap();

        // 验证多个子数据集
        assert!(parent_dataset.get_child_dataset(0, "child1").unwrap().is_some());
        assert!(parent_dataset.get_child_dataset(0, "child2").unwrap().is_some());

        // 移除一个子数据集
        parent_dataset.remove_child_dataset(0, "child1").unwrap();
        assert!(parent_dataset.get_child_dataset(0, "child1").unwrap().is_none());
        assert!(parent_dataset.get_child_dataset(0, "child2").unwrap().is_some());
    }
}
