//! # 列式数据集 (Column Data Set) 模块
//!
//! 这个模块实现了基于列存储的数据集结构，提供了高效的列级数据操作和分析功能。
//! 主要包含两个核心结构体：`RowData` 和 `ColDataSet`。
//!
//! ## 主要特性
//!
//! - **列式存储**：数据按列组织，便于列级别的批量操作和分析
//! - **NULL 值优化**：使用 `Option<T>` 显式处理缺失值
//! - **类型安全**：严格的类型检查和数据完整性验证
//! - **层次结构**：支持行级别的子数据集嵌套
//! - **内存高效**：列式存储适合大数据集的列级操作
//!
//! ## 数据结构
//!
//! ```text
//! ColDataSet {
//!     dataset_id: "users",           // 数据集唯一标识
//!     schema: {                      // 列定义映射
//!         "id" => I32,
//!         "name" => String,
//!         "active" => Bool,
//!     },
//!     columns: {                     // 列数据存储
//!         "id" => NumberArray([Some(1), Some(2), None]),
//!         "name" => StringArray([Some("Alice"), Some("Bob"), None]),
//!         "active" => BoolArray([Some(true), Some(false), None]),
//!     },
//!     children: [Some({...}), None, None]  // 行级子数据集
//! }
//! ```
//!
//! ## 存储方式对比
//!
//! | 特性 | 行式存储 (RowDataSet) | 列式存储 (ColDataSet) |
//! |------|----------------------|----------------------|
//! | 插入性能 | 高 | 中 |
//! | 查询性能 | 行级查询好 | 列级查询好 |
//! | 内存布局 | 连续行 | 连续列 |
//! | 适用场景 | 事务处理 | 数据分析 |
//!
//! ## 使用场景
//!
//! - **数据仓库**：适合大数据集的列级分析和聚合操作
//! - **OLAP 系统**：优化的列式存储适合多维分析
//! - **科学计算**：便于对特定列进行向量化计算
//! - **内存数据库**：高效的列级数据访问和操作

use std::collections::HashMap;

use crate::model::data::cell::CellValue;
use crate::model::data::dataset::ColumnType;

/// 行数据结构，包含列值和子数据集
///
/// `RowData` 表示数据集中的一行数据，采用键值对的方式存储列数据。
/// 支持存储多个列的值以及可选的子数据集，实现复杂的数据关系。
///
/// ## 数据结构
///
/// 每一行包含两部分：
/// - **列值映射**：列名到值的映射，支持所有 `CellValue` 类型
/// - **子数据集映射**：可选的子数据集集合，支持层次化数据结构
///
/// ## 使用示例
///
/// ```rust
/// use cmx_core::model::data::dataset::cds::RowData;
/// use cmx_core::model::data::cell::CellValue;
///
/// // 创建包含多列数据的行
/// let mut row = RowData::new();
/// row.insert("id".to_string(), CellValue::Number(1.into()));
/// row.insert("name".to_string(), CellValue::String("Alice".to_string()));
/// row.insert("active".to_string(), CellValue::Bool(true));
///
/// // 添加子数据集
/// // row.add_child("addresses".to_string(), address_dataset);
/// ```
#[derive(Debug, Clone)]
pub struct RowData {
    /// 列值映射：列名 -> 单元格值
    ///
    /// 使用 HashMap 存储列名到值的映射，提供灵活的列访问方式。
    /// 支持所有 `CellValue` 类型的值存储。
    pub values: HashMap<String, CellValue>,

    /// 子数据集映射：名称 -> 子数据集
    ///
    /// 可选的子数据集集合，用于实现层次化数据结构。
    /// 每个子数据集都有唯一的名称标识，支持复杂的关系型数据。
    /// 这使得一行数据可以包含多个相关的子数据集。
    pub children: Option<HashMap<String, ColDataSet>>,
}

impl RowData {
    /// 创建新的行数据实例
    ///
    /// 创建一个空的行数据对象，初始状态下不包含任何列值和子数据集。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use cmx_core::model::data::dataset::cds::RowData;
    ///
    /// let row = RowData::new();
    /// assert!(row.values.is_empty());
    /// assert!(row.children.is_none());
    /// ```
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            children: None,
        }
    }

    /// 获取指定列的值
    ///
    /// 根据列名查找并返回对应列的值引用。
    /// 如果指定的列不存在，返回 `None`。
    ///
    /// # 参数
    ///
    /// * `key` - 列名标识符
    ///
    /// # 返回值
    ///
    /// 返回 `Option<&CellValue>`：
    /// - `Some(&value)` - 指定列的值引用
    /// - `None` - 指定的列不存在
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::cds::RowData;
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut row = RowData::new();
    /// # row.insert("name".to_string(), CellValue::String("Alice".to_string()));
    ///
    /// if let Some(value) = row.get("name") {
    ///     if let CellValue::String(name) = value {
    ///         println!("Name: {}", name);
    ///     }
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<&CellValue> {
        self.values.get(key)
    }

    /// 插入或更新列值
    ///
    /// 将指定的值插入到指定列中。如果列已存在，会替换原有值。
    ///
    /// # 参数
    ///
    /// * `key` - 列名标识符
    /// * `value` - 要插入的单元格值
    ///
    /// # 行为说明
    ///
    /// - 如果列不存在，会新增列
    /// - 如果列已存在，会替换原有值
    /// - 支持所有有效的 `CellValue` 类型
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::cds::RowData;
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut row = RowData::new();
    ///
    /// // 插入多个列值
    /// row.insert("id".to_string(), CellValue::Number(1.into()));
    /// row.insert("name".to_string(), CellValue::String("Alice".to_string()));
    /// row.insert("active".to_string(), CellValue::Bool(true));
    ///
    /// assert_eq!(row.values.len(), 3);
    /// ```
    pub fn insert(&mut self, key: String, value: CellValue) {
        self.values.insert(key, value);
    }

    /// 获取指定的子数据集
    ///
    /// 根据名称查找并返回对应子数据集的只读引用。
    /// 如果指定的子数据集不存在，返回 `None`。
    ///
    /// # 参数
    ///
    /// * `key` - 子数据集的名称标识符
    ///
    /// # 返回值
    ///
    /// 返回 `Option<&ColDataSet>`：
    /// - `Some(&dataset)` - 指定子数据集的引用
    /// - `None` - 指定的子数据集不存在
    pub fn get_child(&self, key: &str) -> Option<&ColDataSet> {
        self.children.as_ref().and_then(|children| children.get(key))
    }

    /// 获取指定的子数据集的可变引用
    ///
    /// 根据名称查找并返回对应子数据集的可变引用。
    /// 如果指定的子数据集不存在，返回 `None`。
    ///
    /// # 参数
    ///
    /// * `key` - 子数据集的名称标识符
    ///
    /// # 返回值
    ///
    /// 返回 `Option<&mut ColDataSet>`：
    /// - `Some(&mut dataset)` - 指定子数据集的可变引用
    /// - `None` - 指定的子数据集不存在
    pub fn get_child_mut(&mut self, key: &str) -> Option<&mut ColDataSet> {
        self.children.as_mut().and_then(|children| children.get_mut(key))
    }

    /// 添加子数据集
    ///
    /// 将指定的数据集作为子数据集添加到当前行中。
    /// 如果子数据集映射不存在，会自动创建。
    ///
    /// # 参数
    ///
    /// * `key` - 子数据集的唯一名称标识符
    /// * `dataset` - 要添加的子数据集实例
    ///
    /// # 行为说明
    ///
    /// - 如果子数据集映射不存在，会自动创建 HashMap
    /// - 同名的子数据集会被新数据集替换
    /// - 子数据集的所有权会被转移到当前行
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::cds::{RowData, ColDataSet};
    /// # let mut row = RowData::new();
    /// # let child_dataset = ColDataSet::new("addresses".to_string());
    ///
    /// row.add_child("addresses".to_string(), child_dataset);
    /// assert!(row.get_child("addresses").is_some());
    /// ```
    pub fn add_child(&mut self, key: String, dataset: ColDataSet) {
        if self.children.is_none() {
            self.children = Some(HashMap::new());
        }
        self.children.as_mut().unwrap().insert(key, dataset);
    }

    /// 移除指定的子数据集
    ///
    /// 根据名称移除指定的子数据集，并返回被移除的数据集实例。
    /// 如果指定的子数据集不存在，返回 `None`。
    ///
    /// # 参数
    ///
    /// * `key` - 要移除的子数据集的名称标识符
    ///
    /// # 返回值
    ///
    /// 返回 `Option<ColDataSet>`：
    /// - `Some(dataset)` - 被移除的子数据集实例
    /// - `None` - 指定的子数据集不存在
    ///
    /// # 行为说明
    ///
    /// - 被移除的子数据集会从当前行中完全分离
    /// - 返回的数据集实例可以继续使用
    /// - 如果移除后子数据集映射为空，会自动将其设置为 `None` 以节省内存
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::cds::{RowData, ColDataSet};
    /// # let mut row = RowData::new();
    /// # let child_dataset = ColDataSet::new("temp".to_string());
    /// # row.add_child("temp".to_string(), child_dataset);
    ///
    /// let removed = row.remove_child("temp");
    /// assert!(removed.is_some());
    /// assert_eq!(removed.unwrap().dataset_id(), "temp");
    /// assert!(row.get_child("temp").is_none());
    /// ```
    pub fn remove_child(&mut self, key: &str) -> Option<ColDataSet> {
        let result = self.children.as_mut().and_then(|children| children.remove(key));

        // 如果 HashMap 为空，将 children 设置为 None
        if let Some(children) = &self.children {
            if children.is_empty() {
                self.children = None;
            }
        }

        result
    }
}

impl Default for RowData {
    /// 创建默认的行数据实例
    ///
    /// 实现 `Default` trait，返回一个空的行数据对象。
    /// 等价于调用 `RowData::new()`。
    fn default() -> Self {
        Self::new()
    }
}

/// 列数据存储枚举 - 针对 NULL 值优化的列式存储
///
/// `ColumnDataArray` 定义了列数据的存储方式，每种变体对应不同数据类型的列。
/// 使用 `Option<T>` 包装数据类型，能够高效地处理 NULL 值和缺失数据。
///
/// ## 存储策略
///
/// - **StringArray**: 存储字符串类型的列数据
/// - **BoolArray**: 存储布尔类型的列数据
/// - **NumberArray**: 存储所有数值类型的列数据（统一使用 `serde_json::Number`）
///
/// ## NULL 值处理
///
/// 每个数组元素都是 `Option<T>` 类型：
/// - `Some(value)` - 包含实际数据值
/// - `None` - 表示 NULL 值或缺失数据
///
/// ## 内存优化
///
/// 相比于使用特殊标记值（如 0 表示 NULL），`Option<T>` 提供了：
/// - **类型安全**：编译时保证 NULL 值处理正确
/// - **清晰语义**：明确区分有效值和 NULL 值
/// - **性能优化**：Rust 的 `Option<T>` 通常零成本抽象
///
/// ## 使用示例
///
/// ```rust
/// use cmx_core::model::data::dataset::cds::ColumnDataArray;
///
/// // 创建一个包含 NULL 值的字符串数组
/// let string_col = ColumnDataArray::StringArray(vec![
///     Some("Alice".to_string()),
///     None,  // NULL 值
///     Some("Bob".to_string()),
/// ]);
///
/// // 创建数值数组
/// let number_col = ColumnDataArray::NumberArray(vec![
///     Some(1.into()),
///     Some(2.into()),
///     None,  // NULL 值
/// ]);
/// ```
#[derive(Debug, Clone)]
pub enum ColumnDataArray {
    /// 字符串数组 - 存储字符串类型的列数据
    ///
    /// 使用 `Vec<Option<String>>` 存储字符串值，支持 NULL 值。
    /// 适用于存储文本数据、标识符等字符串类型字段。
    StringArray(Vec<Option<String>>),

    /// 布尔数组 - 存储布尔类型的列数据
    ///
    /// 使用 `Vec<Option<bool>>` 存储布尔值，支持 NULL 值。
    /// 适用于存储 true/false 标志、开关状态等布尔类型字段。
    BoolArray(Vec<Option<bool>>),

    /// 数值数组 - 存储所有数值类型的列数据
    ///
    /// 使用 `Vec<Option<serde_json::Number>>` 统一存储所有数值类型。
    /// 支持整数（i8, i16, i32, i64, u8, u16, u32, u64）和浮点数（f32, f64）。
    /// 这是最复杂的数组类型，因为需要处理多种数值类型的统一存储。
    NumberArray(Vec<Option<serde_json::Number>>),
}

impl ColumnDataArray {
    /// 检查指定位置的值是否为 NULL
    ///
    /// 根据索引检查指定位置是否存储了 NULL 值或缺失数据。
    /// 这个方法提供了统一的 NULL 值检查接口。
    ///
    /// # 参数
    ///
    /// * `index` - 要检查的位置索引，从 0 开始
    ///
    /// # 返回值
    ///
    /// 返回 `bool`：
    /// - `true` - 指定位置的值为 NULL 或缺失
    /// - `false` - 指定位置包含有效数据
    ///
    /// # 注意事项
    ///
    /// - 对于数值类型，0 被认为是有效值而不是 NULL
    /// - 索引越界会导致 panic，建议先检查数组长度
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::cds::ColumnDataArray;
    /// # let string_col = ColumnDataArray::StringArray(vec![Some("Alice".to_string()), None]);
    ///
    /// assert_eq!(string_col.is_null(0), false); // 包含 "Alice"
    /// assert_eq!(string_col.is_null(1), true);  // NULL 值
    /// ```
    pub fn is_null(&self, index: usize) -> bool {
        match self {
            ColumnDataArray::BoolArray(vec) => vec[index].is_none(),
            ColumnDataArray::StringArray(vec) => vec[index].is_none(),
            // 数值类型返回 false，因为 0 是有效值
            ColumnDataArray::NumberArray(vec) => vec[index].is_none(),
        }
    }

    /// 将指定位置的值设置为 NULL
    ///
    /// 将指定索引位置的值设置为 NULL，清除该位置存储的任何有效数据。
    /// 这个方法提供了统一的 NULL 值设置接口。
    ///
    /// # 参数
    ///
    /// * `index` - 要设置的位置索引，从 0 开始
    ///
    /// # 行为说明
    ///
    /// - 布尔和字符串类型：将 `Some(value)` 改为 `None`
    /// - 数值类型：将 `Some(number)` 改为 `None`
    /// - 原有的值会被丢弃
    ///
    /// # 注意事项
    ///
    /// - 索引越界会导致 panic
    /// - 对于数值类型，NULL 不同于 0 值
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::cds::ColumnDataArray;
    /// # let mut string_col = ColumnDataArray::StringArray(vec![Some("Alice".to_string())]);
    ///
    /// assert_eq!(string_col.is_null(0), false);
    ///
    /// string_col.set_null(0);
    /// assert_eq!(string_col.is_null(0), true);
    /// ```
    pub fn set_null(&mut self, index: usize) {
        match self {
            ColumnDataArray::BoolArray(vec) => vec[index] = None,
            ColumnDataArray::StringArray(vec) => vec[index] = None,
            // 数值类型设置为 NULL
            ColumnDataArray::NumberArray(vec) => vec[index] = None,
        }
    }
}

/// 列式存储的数据集结构
///
/// `ColDataSet` 是基于列存储的数据集实现，提供了高效的列级数据操作和分析功能。
/// 数据按列组织，每列使用专门的数组存储，能够提供优秀的列级查询和批量操作性能。
///
/// ## 核心特性
///
/// - **列式存储**：数据按列连续存储，便于列级别的向量化操作
/// - **内存效率**：适合大数据集的列级分析和聚合计算
/// - **NULL 值优化**：使用 `Option<T>` 显式处理缺失数据
/// - **类型安全**：严格的类型检查和数据完整性验证
/// - **层次结构**：支持行级别的子数据集嵌套
/// - **数据一致性**：自动验证和维护所有列的长度一致性
///
/// ## 数据结构详解
///
/// ```text
/// ColDataSet {
///     dataset_id: "sales_data",        // 数据集唯一标识
///     schema: {                        // 列模式定义
///         "product_id" => I32,
///         "product_name" => String,
///         "price" => F64,
///         "in_stock" => Bool,
///     },
///     columns: {                       // 列数据存储
///         "product_id" => NumberArray([Some(1), Some(2), Some(3)]),
///         "product_name" => StringArray([Some("Apple"), Some("Banana"), None]),
///         "price" => NumberArray([Some(1.50), Some(0.75), Some(2.00)]),
///         "in_stock" => BoolArray([Some(true), Some(false), Some(true)]),
///     },
///     children: [None, None, Some({...})]  // 每行的子数据集（可选）
/// }
/// ```
///
/// ## 存储优势
///
/// ### 相比行式存储的优势：
/// - **列级查询**：直接访问整个列，适合聚合和分析操作
/// - **压缩友好**：相同类型的连续存储有利于压缩算法
/// - **向量化优化**：便于 SIMD 指令优化
/// - **缓存效率**：连续内存访问模式对 CPU 缓存更友好
///
/// ### 适用场景：
/// - **数据仓库**：OLAP 分析、报表生成、大数据处理
/// - **科学计算**：数值计算、统计分析、机器学习特征处理
/// - **时间序列**：时序数据分析、趋势计算
/// - **日志分析**：大规模日志数据的列式聚合分析
///
/// ## 设计权衡
///
/// ### 优势：
/// - ✅ 优秀的列级查询性能
/// - ✅ 内存访问模式优化
/// - ✅ 便于数据压缩和编码
/// - ✅ 适合分析型工作负载
///
/// ### 权衡：
/// - ⚠️ 行级插入/更新相对较慢
/// - ⚠️ 单行查询需要组合多个列
/// - ⚠️ 内存布局较为复杂
/// - ⚠️ 不适合频繁的单行操作
///
/// ## 内存布局
///
/// ```text
/// 行式存储:    [row1: {id, name, price}] [row2: {id, name, price}] ...
/// 列式存储:    id: [1, 2, 3, ...] name: ["A", "B", "C", ...] price: [1.0, 2.0, 3.0, ...]
/// ```
///
/// ## 线程安全
///
/// 当前实现不是线程安全的。如果需要在多线程环境中使用，
/// 需要额外的同步机制或使用线程安全的变体。
#[derive(Debug, Clone)]
pub struct ColDataSet {
    /// 数据集唯一标识符
    ///
    /// 用于区分不同的数据集实例，在整个系统中应该是唯一的。
    /// 支持字母、数字、下划线等字符，通常用于数据集的逻辑标识。
    pub dataset_id: String,

    /// 列模式定义：列名 -> 列类型
    ///
    /// 存储数据集的模式信息，将列名映射到对应的数据类型。
    /// 这是数据完整性验证和类型检查的基础。
    /// 一旦数据集创建后，模式通常是只读的。
    pub schema: HashMap<String, ColumnType>,

    /// 列数据存储：列名 -> 列数据数组
    ///
    /// 核心数据存储结构，每列使用专门的数组存储。
    /// 所有列的数组长度必须保持一致，这是数据一致性的关键。
    pub columns: HashMap<String, ColumnDataArray>,

    /// 子数据集存储：每行可选择性包含子数据集
    ///
    /// 为每一行数据提供可选的子数据集支持，实现层次化数据结构。
    /// 数组长度与行数一致，每个元素都是可选的子数据集映射。
    /// 支持复杂的数据关系和嵌套结构。
    pub children: Vec<Option<HashMap<String, ColDataSet>>>,
}

impl ColDataSet {
    /// 创建新的列式数据集实例
    ///
    /// 使用指定的数据集ID创建空的列式数据集。初始状态下不包含任何列定义、数据和子数据集。
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
    /// use cmx_core::model::data::dataset::cds::ColDataSet;
    ///
    /// let dataset = ColDataSet::new("sales_data".to_string());
    /// assert_eq!(dataset.dataset_id(), "sales_data");
    /// assert_eq!(dataset.row_count(), 0);
    /// assert!(dataset.columns.is_empty());
    /// ```
    pub fn new(dataset_id: String) -> Self {
        Self {
            dataset_id,
            schema: HashMap::new(),
            columns: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// 获取数据集的行数量
    ///
    /// 返回当前数据集中存储的数据行数量。由于列式存储的特性，
    /// 所有列的长度都必须保持一致，因此可以从任意一列获取长度。
    ///
    /// # 返回值
    ///
    /// 返回行数量：
    /// - 如果数据集包含列，返回列的长度
    /// - 如果数据集为空（没有列），返回 0
    ///
    /// # 实现说明
    ///
    /// 这个方法利用了列式存储的数据一致性特性：
    /// 所有列的数组长度都相同，因此只需要检查任意一列即可。
    /// 这种设计保证了数据完整性，同时提供了高效的长度查询。
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{cds::ColDataSet, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = ColDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # let mut row = dataset::cds::RowData::new();
    /// # row.insert("id".to_string(), CellValue::Number(1.into()));
    /// # dataset.append_row(&row).unwrap();
    ///
    /// assert_eq!(dataset.row_count(), 1);
    /// ```
    ///
    /// # 性能特性
    ///
    /// - **时间复杂度**: O(1) - HashMap 查找 + 数组长度获取
    /// - **空间复杂度**: O(1) - 不分配额外内存
    pub fn row_count(&self) -> usize {
        // 从任意一列获取长度即可，因为所有列的长度都是相同的
        self.columns.values().next()
            .map(|col| match col {
                ColumnDataArray::BoolArray(vec) => vec.len(),
                ColumnDataArray::StringArray(vec) => vec.len(),
                ColumnDataArray::NumberArray(vec) => vec.len(),
                // ColumnDataArray::CHILD(vec) => vec.len(),
            })
            .unwrap_or(0) // 如果没有列，返回 0
    }

    /// 验证所有列的长度是否一致
    ///
    /// 检查数据集中的所有列是否具有相同的长度，以及子数据集数组的长度是否正确。
    /// 这是维护数据完整性的关键方法，确保列式存储的一致性。
    ///
    /// # 验证内容
    ///
    /// 1. **列长度一致性**：所有列的数组长度必须相同
    /// 2. **子数据集长度**：子数据集数组长度必须等于行数
    /// 3. **数据完整性**：确保没有出现长度不匹配的情况
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), ColDataSetError>`：
    /// - `Ok(())` - 所有长度都一致，数据完整性良好
    /// - `Err(ColDataSetError::InconsistentLength)` - 发现长度不一致的情况
    ///
    /// # 错误情况
    ///
    /// - 如果某一列的长度与其他列不一致
    /// - 如果子数据集数组长度与行数不匹配
    /// - 如果数据结构出现损坏
    ///
    /// # 设计理念
    ///
    /// 列式存储的核心要求是所有列必须保持相同的长度。
    /// 这个方法在每次数据修改操作后都会被调用，确保数据一致性。
    /// 这种设计避免了行式存储中可能出现的稀疏矩阵问题。
    ///
    /// # 性能考虑
    ///
    /// - 在大数据集上，这个方法会遍历所有列
    /// - 建议在调试模式下频繁调用，在生产环境中按需调用
    fn validate_row_counts(&self) -> Result<(), ColDataSetError> {
        if self.columns.is_empty() {
            return Ok(());
        }

        let expected_len = self.row_count();

        // Also validate children length
        if self.children.len() != expected_len {
            return Err(ColDataSetError::InconsistentLength {
                column: "children".to_string(),
                expected: expected_len,
                actual: self.children.len(),
            });
        }

        for (col_name, col_data) in &self.columns {
            let actual_len = match col_data {
                ColumnDataArray::BoolArray(vec) => vec.len(),
                ColumnDataArray::StringArray(vec) => vec.len(),
                ColumnDataArray::NumberArray(vec) => vec.len(),
                // ColumnDataArray::CHILD(vec) => vec.len(),
            };
            if actual_len != expected_len {
                return Err(ColDataSetError::InconsistentLength {
                    column: col_name.clone(),
                    expected: expected_len,
                    actual: actual_len,
                });
            }
        }
        Ok(())
    }

    /// 添加列定义到数据集模式
    ///
    /// 为数据集添加新的列定义，包括列名和数据类型。新添加的列会被初始化为适当的默认值数组。
    /// 这是构建数据集模式的核心方法，必须在添加数据之前完成。
    ///
    /// # 参数
    ///
    /// * `name` - 列的唯一名称标识符，不能与现有列重复
    /// * `col_type` - 列的数据类型，定义了该列可以存储的数据类型
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), ColDataSetError>`：
    /// - `Ok(())` - 列添加成功
    /// - `Err(ColDataSetError::InconsistentLength)` - 如果数据一致性检查失败（通常不会发生）
    ///
    /// # 行为说明
    ///
    /// 1. **创建列数据数组**：根据列类型创建相应长度的数组，初始值为 NULL
    /// 2. **更新模式定义**：将列名和类型添加到模式映射中
    /// 3. **验证一致性**：确保所有列长度保持一致
    ///
    /// # 类型映射规则
    ///
    /// - `Bool` → `BoolArray` (布尔数组)
    /// - `String` → `StringArray` (字符串数组)
    /// - 所有数字类型 → `NumberArray` (数值数组)
    ///
    /// # 重要限制
    ///
    /// - **列名唯一性**：不能添加同名列（当前实现未检查，建议后续改进）
    /// - **顺序相关性**：建议在添加数据前完成所有列定义
    /// - **不可逆操作**：添加列后无法直接移除
    ///
    /// # 示例
    ///
    /// ```rust
    /// use cmx_core::model::data::dataset::{cds::ColDataSet, ColumnType};
    ///
    /// let mut dataset = ColDataSet::new("products".to_string());
    ///
    /// // 添加多列定义
    /// dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// dataset.add_column("name".to_string(), ColumnType::String).unwrap();
    /// dataset.add_column("price".to_string(), ColumnType::F64).unwrap();
    /// dataset.add_column("in_stock".to_string(), ColumnType::Bool).unwrap();
    ///
    /// assert_eq!(dataset.schema.len(), 4);
    /// ```
    ///
    /// # 内存影响
    ///
    /// 添加列会为现有所有行创建新的数组元素。如果数据集很大，
    /// 这个操作可能会消耗显著的内存。
    pub fn add_column(&mut self, name: String, col_type: ColumnType) -> Result<(), ColDataSetError> {
        let current_row_count = self.row_count();

        // 根据类型创建对应的空向量，并填充到当前行数
        let column_data = match col_type {
            ColumnType::Bool => ColumnDataArray::BoolArray(vec![None; current_row_count]),
            ColumnType::String => ColumnDataArray::StringArray(vec![None; current_row_count]),
            ColumnType::I8 | ColumnType::I16 | ColumnType::I32 | ColumnType::I64 |
            ColumnType::U8 | ColumnType::U16 | ColumnType::U32 | ColumnType::U64 |
            ColumnType::F32 | ColumnType::F64 => ColumnDataArray::NumberArray(vec![None; current_row_count]),
        };

        self.schema.insert(name.clone(), col_type);
        self.columns.insert(name, column_data);

        self.validate_row_counts()
    }

    /// 在数据集末尾添加一行数据
    ///
    /// 将包含完整列值的数据行添加到数据集的末尾。这是列式存储的核心数据插入方法。
    /// 数据会被分散存储到对应的列数组中，实现了列式存储的本质。
    ///
    /// # 参数
    ///
    /// * `row` - 包含列值的行数据引用，必须包含数据集定义的所有列
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), ColDataSetError>`：
    /// - `Ok(())` - 行数据添加成功
    /// - `Err(ColDataSetError::TypeMismatch)` - 数据类型与列定义不匹配
    /// - `Err(ColDataSetError::InconsistentLength)` - 数据一致性检查失败
    ///
    /// # 数据转换过程
    ///
    /// ```text
    /// 输入: RowData { values: {"id": Number(1), "name": String("Alice")} }
    ///
    /// 处理后:
    /// columns["id"] NumberArray: [...Some(1)]
    /// columns["name"] StringArray: [...Some("Alice")]
    /// ```
    ///
    /// # 类型匹配规则
    ///
    /// | RowData 值类型 | 列数组类型 | 处理方式 |
    /// |---------------|-----------|---------|
    /// | `CellValue::Bool` | `BoolArray` | 直接存储 |
    /// | `CellValue::String` | `StringArray` | 直接存储 |
    /// | `CellValue::Number` | `NumberArray` | 直接存储 |
    /// | `CellValue::Null` | 任何类型 | 存储为 `None` |
    /// | 其他 | - | 类型不匹配错误 |
    ///
    /// # 子数据集处理
    ///
    /// - 如果行数据包含子数据集，会被完整复制到数据集的子数据集数组中
    /// - 如果行数据没有子数据集，对应位置存储 `None`
    ///
    /// # 性能特点
    ///
    /// - **时间复杂度**: O(C) - C 为列数量，需要更新每一列
    /// - **空间复杂度**: O(C) - 每列添加一个元素
    /// - **内存操作**: 高效的列式追加，避免了行式存储的内存重新分配
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{cds::{ColDataSet, RowData}, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = ColDataSet::new("users".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_column("name".to_string(), ColumnType::String).unwrap();
    ///
    /// // 创建行数据
    /// let mut row = RowData::new();
    /// row.insert("id".to_string(), CellValue::Number(1.into()));
    /// row.insert("name".to_string(), CellValue::String("Alice".to_string()));
    ///
    /// // 添加到数据集
    /// dataset.append_row(&row).unwrap();
    /// assert_eq!(dataset.row_count(), 1);
    /// ```
    ///
    /// # 错误处理
    ///
    /// - **缺失列**: 如果行数据缺少某个列，会使用 `CellValue::Null` 填充
    /// - **类型不匹配**: 如果值的类型与列定义不匹配，返回类型错误
    /// - **数据损坏**: 如果操作导致数据不一致，会在验证阶段检测出来
    ///
    /// # 设计优势
    ///
    /// 这种列式插入方式的优势：
    /// - ✅ **缓存友好**: 连续内存写入
    /// - ✅ **向量化就绪**: 便于后续的 SIMD 操作
    /// - ✅ **压缩优化**: 有利于列级别的压缩算法
    pub fn append_row(&mut self, row: &RowData) -> Result<(), ColDataSetError> {
        self.validate_row_counts()?;

        // 执行添加操作
        for (col_name, col_data) in &mut self.columns {
            let value = row.get(col_name).cloned().unwrap_or(CellValue::Null);

            match (col_data, value) {
                // Bool 和 String 保持不变
                (ColumnDataArray::BoolArray(vec), CellValue::Bool(v)) => {
                    vec.push(Some(v));
                }
                (ColumnDataArray::BoolArray(vec), CellValue::Null) => {
                    vec.push(None);
                }
                (ColumnDataArray::StringArray(vec), CellValue::String(v)) => {
                    vec.push(Some(v));
                }
                (ColumnDataArray::StringArray(vec), CellValue::Null) => {
                    vec.push(None);
                }

                // 数值类型的处理：从 Number 中提取值，NULL 设为 0
                (ColumnDataArray::NumberArray(vec), CellValue::Number(v)) => {
                    vec.push(Some(v.clone()));
                }
                (ColumnDataArray::NumberArray(vec), CellValue::Null) => {
                    vec.push(None);
                }

                // ... 其他数值类型类似处理

                // 类型不匹配的情况
                (col_data, value) => {
                    return Err(ColDataSetError::TypeMismatch {
                        column: col_name.clone(),
                        expected: format!("{:?}", col_data),
                        actual: format!("{:?}", value),
                    });
                }
            }
        }

        // Add children data
        if let Some(row_children) = &row.children {
            self.children.push(Some(row_children.clone()));
        } else {
            self.children.push(None);
        }

        self.validate_row_counts()
    }

    /// 获取指定列的数据数组引用
    ///
    /// 根据列名返回对应列的完整数据数组引用，提供对列级数据的直接访问。
    /// 这是进行列级数据分析和处理的基础方法。
    ///
    /// # 参数
    ///
    /// * `name` - 要获取的列名
    ///
    /// # 返回值
    ///
    /// 返回 `Option<&ColumnDataArray>`：
    /// - `Some(&array)` - 指定列的数据数组引用
    /// - `None` - 指定的列不存在
    ///
    /// # 使用场景
    ///
    /// - **列级分析**: 对整个列进行统计计算
    /// - **数据导出**: 提取特定列的数据
    /// - **批量操作**: 对列数据进行向量化处理
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::cds::ColDataSet;
    /// # let dataset = ColDataSet::new("test".to_string());
    ///
    /// if let Some(column_data) = dataset.get_column("price") {
    ///     match column_data {
    ///         cmx_core::model::data::dataset::cds::ColumnDataArray::NumberArray(vec) => {
    ///             let prices: Vec<f64> = vec.iter()
    ///                 .filter_map(|v| v.as_ref().and_then(|n| n.as_f64()))
    ///                 .collect();
    ///             println!("Prices: {:?}", prices);
    ///         }
    ///         _ => println!("Not a number column"),
    ///     }
    /// }
    /// ```
    ///
    /// # 性能说明
    ///
    /// - **时间复杂度**: O(1) - HashMap 查找
    /// - **内存效率**: 返回引用，不复制数据
    /// - **线程安全**: 只读访问是线程安全的
    pub fn get_column(&self, name: &str) -> Option<&ColumnDataArray> {
        self.columns.get(name)
    }

    /// 在指定位置插入一行数据
    ///
    /// 将数据行插入到数据集的指定位置。插入位置之后的所有行会向后移动，
    /// 为新行腾出空间。这是列式存储中的复杂操作，需要同时更新所有列。
    ///
    /// # 参数
    ///
    /// * `index` - 插入位置的索引，从 0 开始，可以等于当前行数（追加）
    /// * `row` - 要插入的行数据引用
    ///
    /// # 返回值
    ///
    /// 返回 `Result<(), ColDataSetError>`：
    /// - `Ok(())` - 行插入成功
    /// - `Err(ColDataSetError::IndexOutOfBounds)` - 索引超出有效范围
    /// - `Err(ColDataSetError::TypeMismatch)` - 数据类型不匹配
    ///
    /// # 行为说明
    ///
    /// 1. **位置验证**: 检查插入位置的有效性
    /// 2. **数据一致性**: 验证所有列的数据类型匹配
    /// 3. **批量插入**: 同时在所有列的对应位置插入数据
    /// 4. **子数据集处理**: 在子数据集数组中插入相应的子数据集
    ///
    /// # 性能特点
    ///
    /// - **时间复杂度**: O(C × N) - C 为列数，N 为受影响的行数
    /// - **空间复杂度**: O(C) - 每列添加一个元素
    /// - **最坏情况**: 在开头插入时，所有现有数据都需要移动
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{cds::{ColDataSet, RowData}, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = ColDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # dataset.add_row(RowData::new()).unwrap(); // 添加一行作为基准
    ///
    /// let mut new_row = RowData::new();
    /// new_row.insert("id".to_string(), CellValue::Number(100.into()));
    ///
    /// // 在开头插入
    /// dataset.insert_row(0, &new_row).unwrap();
    /// assert_eq!(dataset.row_count(), 2);
    /// ```
    ///
    /// # 注意事项
    ///
    /// - **性能影响**: 中间位置插入会移动大量数据
    /// - **索引调整**: 插入操作会改变后续所有行的索引
    /// - **内存重新分配**: 可能触发向量扩容
    pub fn insert_row(&mut self, index: usize, row: &RowData) -> Result<(), ColDataSetError> {
        self.validate_row_counts()?;

        if index > self.row_count() {
            return Err(ColDataSetError::IndexOutOfBounds {
                index: index,
                row_count: self.row_count(),
            });
        }

        // 为每一列插入数据
        for (col_name, col_data) in &mut self.columns {
            let value = row.values.get(col_name).cloned().unwrap_or(CellValue::Null);

            match (col_data, value) {
                // Bool 类型处理
                (ColumnDataArray::BoolArray(vec), CellValue::Bool(v)) => {
                    vec.insert(index, Some(v));
                }
                (ColumnDataArray::BoolArray(vec), CellValue::Null) => {
                    vec.insert(index, None);
                }

                // String 类型处理
                (ColumnDataArray::StringArray(vec), CellValue::String(v)) => {
                    vec.insert(index, Some(v));
                }
                (ColumnDataArray::StringArray(vec), CellValue::Null) => {
                    vec.insert(index, None);
                }

                // 数值类型处理：从 Number 中提取值
                (ColumnDataArray::NumberArray(vec), CellValue::Number(v)) => {
                    vec.insert(index, Some(v.clone()));
                }
                (ColumnDataArray::NumberArray(vec), CellValue::Null) => {
                    vec.insert(index, None);
                }
                // ... 其他数值类型类似处理

                // 类型不匹配的情况
                (col_data, value) => {
                    return Err(ColDataSetError::TypeMismatch {
                        column: col_name.clone(),
                        expected: format!("{:?}", col_data),
                        actual: format!("{:?}", value),
                    });
                }
            }
        }

        // Insert children data
        if let Some(row_children) = &row.children {
            self.children.insert(index, Some(row_children.clone()));
        } else {
            self.children.insert(index, None);
        }

        self.validate_row_counts()
    }

    /// 删除指定行并返回被删除的行数据
    ///
    /// 根据行索引删除数据集中的指定行，并将删除的行数据重新组装后返回。
    /// 删除位置之后的所有行会向前移动，填补空隙。
    ///
    /// # 参数
    ///
    /// * `index` - 要删除的行索引，从 0 开始
    ///
    /// # 返回值
    ///
    /// 返回 `Result<RowData, ColDataSetError>`：
    /// - `Ok(row)` - 被删除的行数据，包含所有列值和子数据集
    /// - `Err(ColDataSetError::IndexOutOfBounds)` - 索引超出有效范围
    ///
    /// # 数据重组过程
    ///
    /// ```text
    /// 删除前: columns["id"] = [1, 2, 3], columns["name"] = ["A", "B", "C"]
    /// 删除索引 1: 从每列移除索引 1 的元素
    /// 删除后: columns["id"] = [1, 3], columns["name"] = ["A", "C"]
    /// 返回: RowData { "id": 2, "name": "B" }
    /// ```
    ///
    /// # 性能特点
    ///
    /// - **时间复杂度**: O(C × N) - C 为列数，N 为受影响的行数
    /// - **最坏情况**: 删除开头行时，所有后续数据都需要移动
    /// - **内存操作**: 向量元素移除，可能会触发内存重新分配
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::{cds::{ColDataSet, RowData}, ColumnType};
    /// # use cmx_core::model::data::cell::CellValue;
    /// # let mut dataset = ColDataSet::new("test".to_string());
    /// # dataset.add_column("id".to_string(), ColumnType::I32).unwrap();
    /// # let mut row = RowData::new();
    /// # row.insert("id".to_string(), CellValue::Number(42.into()));
    /// # dataset.append_row(&row).unwrap();
    ///
    /// // 删除行并获取删除的数据
    /// let removed_row = dataset.remove_row(0).unwrap();
    /// if let Some(id_value) = removed_row.get("id") {
    ///     println!("Removed ID: {:?}", id_value);
    /// }
    /// ```
    ///
    /// # 重要说明
    ///
    /// - **数据完整性**: 返回的行数据包含删除前该行的完整状态
    /// - **子数据集**: 如果行包含子数据集，也会被包含在返回的行数据中
    /// - **不可逆操作**: 删除操作无法撤销，除非保存返回的行数据
    pub fn remove_row(&mut self, index: usize) -> Result<RowData, ColDataSetError> {
        self.validate_row_counts()?;

        if index >= self.row_count() {
            return Err(ColDataSetError::IndexOutOfBounds {
                index: index,
                row_count: self.row_count(),
            });
        }

        let mut removed_row = RowData::new();

        // 从每一列中删除对应行的数据并收集被删除的值
        for (col_name, col_data) in &mut self.columns {
            let value = match col_data {
                ColumnDataArray::BoolArray(vec) => {
                    let v = vec.remove(index);
                    v.map_or(CellValue::Null, CellValue::Bool)
                }
                ColumnDataArray::StringArray(vec) => {
                    let v = vec.remove(index);
                    v.map_or(CellValue::Null, CellValue::String)
                }
                ColumnDataArray::NumberArray(vec) => {
                    let v = vec.remove(index);
                    v.map_or(CellValue::Null, CellValue::Number)
                }
                // ColumnDataArray::CHILD(vec) => {
                //     let v = vec.remove(index);
                //     v.map_or(CellValue::Null, CellValue::Child)
                // }
            };
            removed_row.insert(col_name.clone(), value);
        }

        // Remove and set children data
        if let Some(children) = self.children.remove(index) {
            removed_row.children = Some(children);
        }

        self.validate_row_counts()?;
        Ok(removed_row)
    }

    /// 获取数据集的ID标识符
    ///
    /// 返回数据集的唯一标识符字符串引用。这个ID在数据集生命周期内是只读的。
    ///
    /// # 返回值
    ///
    /// 返回数据集ID的字符串切片引用
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use cmx_core::model::data::dataset::cds::ColDataSet;
    /// # let dataset = ColDataSet::new("sales_2024".to_string());
    ///
    /// assert_eq!(dataset.dataset_id(), "sales_2024");
    /// ```
    pub fn dataset_id(&self) -> &str {
        &self.dataset_id
    }
}

/// 列式数据集操作错误类型
///
/// 定义了列式数据集在各种操作过程中可能出现的错误情况。
/// 这些错误提供了详细的上下文信息，帮助诊断和处理数据操作问题。
#[derive(Debug)]
pub enum ColDataSetError {
    /// 列长度不一致错误
    ///
    /// 当数据集中的列具有不同的长度时发生，通常表示数据损坏或操作错误。
    /// 列式存储要求所有列必须具有相同的长度。
    InconsistentLength {
        /// 出现问题的列名
        column: String,
        /// 期望的长度
        expected: usize,
        /// 实际的长度
        actual: usize,
    },

    /// 索引超出边界错误
    ///
    /// 当尝试访问不存在的行索引时发生。
    /// 索引必须在 0 到 (row_count - 1) 的范围内。
    IndexOutOfBounds {
        /// 尝试访问的索引
        index: usize,
        /// 数据集的总行数
        row_count: usize,
    },

    /// 类型不匹配错误
    ///
    /// 当插入的数据类型与列定义的类型不匹配时发生。
    /// 例如，尝试向字符串列插入数字值。
    TypeMismatch {
        /// 出现问题的列名
        column: String,
        /// 期望的数据类型描述
        expected: String,
        /// 实际的数据类型描述
        actual: String,
    },
}

impl std::fmt::Display for ColDataSetError {
    /// 格式化错误信息为用户友好的字符串
    ///
    /// 为每种错误类型提供清晰、详细的错误描述，
    /// 包含相关的上下文信息以便调试。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColDataSetError::InconsistentLength { column, expected, actual } => {
                write!(f, "Column '{}' has inconsistent length: expected {}, got {}",
                    column, expected, actual)
            }
            ColDataSetError::IndexOutOfBounds { index, row_count } => {
                write!(f, "Index {} out of bounds (row count: {})", index, row_count)
            }
            ColDataSetError::TypeMismatch { column, expected, actual } => {
                write!(f, "Type mismatch for column {}: expected {}, got {}",
                    column, expected, actual)
            }
        }
    }
}

impl std::error::Error for ColDataSetError {
    /// 实现标准错误特征
    ///
    /// 使 ColDataSetError 符合 Rust 标准错误处理生态系统，
    /// 可以与其他错误处理库和框架集成使用。
}

/// 单元测试模块
///
/// 包含列式数据集的完整测试用例集合，验证所有核心功能和边界条件。
/// 测试覆盖了基本操作、错误处理、数据一致性、性能边界等各个方面。
///
/// ## 测试分类
///
/// - **基础功能测试**: 数据集创建、列定义、数据插入
/// - **边界条件测试**: 索引越界、类型不匹配、空值处理
/// - **数据一致性测试**: 长度验证、类型检查
/// - **子数据集测试**: 层次化数据结构操作
/// - **错误处理测试**: 各种异常情况的正确处理
#[cfg(test)]
mod tests {
    use super::*;

    /// 基础数据集功能测试
    ///
    /// 测试数据集的基本创建、列定义、数据插入和查询功能。
    /// 这是最核心的功能测试，确保数据集的基本操作正常工作。
    #[test]
    fn test_dataset() {
        let mut dataset = ColDataSet::new("test".into());

        // 添加列定义
        dataset.add_column("id".into(), ColumnType::I32).unwrap();
        dataset.add_column("name".into(), ColumnType::String).unwrap();
        dataset.add_column("score".into(), ColumnType::F64).unwrap();
        dataset.add_column("active".into(), ColumnType::Bool).unwrap();

        // 添加行数据
        let mut row1 = RowData::new();
        row1.insert("id".into(), CellValue::Number(1.into()));
        row1.insert("name".into(), CellValue::String("Alice".into()));
        row1.insert("score".into(), CellValue::Number(serde_json::Number::from_f64(95.5).unwrap()));
        row1.insert("active".into(), CellValue::Bool(true));
        dataset.append_row(&row1).unwrap();

        // 添加包含空值的行
        let mut row2 = RowData::new();
        row2.insert("id".into(), CellValue::Number(2.into()));
        row2.insert("name".into(), CellValue::String("Bob".into()));
        row2.insert("score".into(), CellValue::Null);
        row2.insert("active".into(), CellValue::Bool(false));
        dataset.append_row(&row2).unwrap();

        // 验证数据
        match dataset.get_column("id").unwrap() {
            ColumnDataArray::NumberArray(vec) => {
                assert_eq!(vec[0].as_ref().unwrap().as_i64().unwrap(), 1);
                assert_eq!(vec[1].as_ref().unwrap().as_i64().unwrap(), 2);
            }
            _ => panic!("Wrong type"),
        }

        match dataset.get_column("score").unwrap() {
            ColumnDataArray::NumberArray(vec) => {
                assert_eq!(vec[0].as_ref().unwrap().as_f64().unwrap(), 95.5);
                assert!(vec[1].is_none()); // NULL value
            }
            _ => panic!("Wrong type"),
        }
    }

    /// 类型不匹配错误测试
    ///
    /// 验证当插入的数据类型与列定义不匹配时，系统能够正确检测并报告错误。
    /// 这确保了数据类型的一致性和安全性。
    #[test]
    fn test_type_mismatch() {
        let mut dataset = ColDataSet::new("test".into());
        dataset.add_column("id".into(), ColumnType::I32).unwrap();

        let mut row = RowData::new();
        row.insert("id".into(), CellValue::String("not a number".into()));
        
        // 应该返回错误
        assert!(dataset.append_row(&row).is_err());
    }

    /// 行插入和删除功能测试
    ///
    /// 验证在数据集中间位置插入和删除行的功能。
    /// 测试索引调整、数据移动和删除结果返回的正确性。
    #[test]
    fn test_insert_and_remove_row() {
        let mut dataset = ColDataSet::new("test".into());

        // 添加列定义
        dataset.add_column("id".into(), ColumnType::I32).unwrap();
        dataset.add_column("name".into(), ColumnType::String).unwrap();

        // 添加初始行
        let mut row1 = RowData::new();
        row1.insert("id".into(), CellValue::Number(1.into()));
        row1.insert("name".into(), CellValue::String("Alice".into()));
        dataset.append_row(&row1).unwrap();

        let mut row2 = RowData::new();
        row2.insert("id".into(), CellValue::Number(3.into()));
        row2.insert("name".into(), CellValue::String("Charlie".into()));
        dataset.append_row(&row2).unwrap();

        // 在中间插入一行
        let mut row_insert = RowData::new();
        row_insert.insert("id".into(), CellValue::Number(2.into()));
        row_insert.insert("name".into(), CellValue::String("Bob".into()));
        dataset.insert_row(1, &row_insert).unwrap();

        // 验证插入结果
        match dataset.get_column("id").unwrap() {
            ColumnDataArray::NumberArray(vec) => {
                assert_eq!(vec[0].as_ref().unwrap().as_i64().unwrap(), 1);
                assert_eq!(vec[1].as_ref().unwrap().as_i64().unwrap(), 2);
                assert_eq!(vec[2].as_ref().unwrap().as_i64().unwrap(), 3);
            }
            _ => panic!("Wrong type"),
        }

        // 删除中间的行
        dataset.remove_row(1).unwrap();

        // 验证删除结果
        match dataset.get_column("id").unwrap() {
            ColumnDataArray::NumberArray(vec) => {
                assert_eq!(vec[0].as_ref().unwrap().as_i64().unwrap(), 1);
                assert_eq!(vec[1].as_ref().unwrap().as_i64().unwrap(), 3);
                assert_eq!(vec.len(), 2);
            }
            _ => panic!("Wrong type"),
        }

        // 验证行数
        assert_eq!(dataset.row_count(), 2);
    }

    #[test]
    fn test_invalid_operations() {
        let mut dataset = ColDataSet::new("test".into());
        dataset.add_column("id".into(), ColumnType::I32).unwrap();

        // 测试在无效位置插入
        let row = RowData::new();
        assert!(dataset.insert_row(1, &row).is_err());

        // 测试删除无效行
        assert!(dataset.remove_row(0).is_err());
    }

    /// 删除行返回值测试
    ///
    /// 验证删除行操作返回的行数据是否完整且正确。
    /// 确保被删除的行数据包含所有列值和可能的子数据集。
    #[test]
    fn test_remove_row_return_value() {
        let mut dataset = ColDataSet::new("test".into());

        // 添加列定义
        dataset.add_column("id".into(), ColumnType::I32).unwrap();
        dataset.add_column("name".into(), ColumnType::String).unwrap();
        dataset.add_column("score".into(), ColumnType::F64).unwrap();

        // 添加测试数据
        let mut row1 = RowData::new();
        row1.insert("id".into(), CellValue::Number(1.into()));
        row1.insert("name".into(), CellValue::String("Alice".into()));
        row1.insert("score".into(), CellValue::Number(serde_json::Number::from_f64(95.5).unwrap()));
        dataset.append_row(&row1).unwrap();

        let mut row2 = RowData::new();
        row2.insert("id".into(), CellValue::Number(2.into()));
        row2.insert("name".into(), CellValue::String("Bob".into()));
        row2.insert("score".into(), CellValue::Null);  // NULL value
        dataset.append_row(&row2).unwrap();

        // 删除
        let removed_row = dataset.remove_row(1).unwrap();

        // 验证删除结果
        match removed_row.get("id").unwrap() {
            CellValue::Number(v) => assert_eq!(v.as_i64().unwrap(), 2),
            _ => panic!("Wrong type"),
        }

        match removed_row.get("name").unwrap() {
            CellValue::String(v) => assert_eq!(v, "Bob"),
            _ => panic!("Wrong type"),
        }

        match removed_row.get("score").unwrap() {
            CellValue::Null => (),
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_row_count() {
        let mut dataset = ColDataSet::new("test".into());
        assert_eq!(dataset.row_count(), 0);

        // 添加列定义
        dataset.add_column("id".into(), ColumnType::I32).unwrap();
        dataset.add_column("name".into(), ColumnType::String).unwrap();

        // 添加行数据
        let mut row1 = RowData::new();
        row1.insert("id".into(), CellValue::Number(1.into()));
        row1.insert("name".into(), CellValue::String("Alice".into()));
        dataset.append_row(&row1).unwrap();
        assert_eq!(dataset.row_count(), 1);

        // 添加第二行
        let mut row2 = RowData::new();
        row2.insert("id".into(), CellValue::Number(2.into()));
        row2.insert("name".into(), CellValue::String("Bob".into()));
        dataset.append_row(&row2).unwrap();
        assert_eq!(dataset.row_count(), 2);

        // 删除一行
        dataset.remove_row(0).unwrap();
        assert_eq!(dataset.row_count(), 1);
    }

    #[test]
    fn test_column_consistency() {
        let mut dataset = ColDataSet::new("test".into());

        // 添加初始列
        dataset.add_column("id".into(), ColumnType::I32).unwrap();
        
        // 添加一行数据
        let mut row = RowData::new();
        row.insert("id".into(), CellValue::Number(1.into()));
        dataset.append_row(&row).unwrap();

        // 添加新列，应该自动填充现有行数的空值
        dataset.add_column("name".into(), ColumnType::String).unwrap();

        // 验证所有列长度一致
        assert_eq!(dataset.row_count(), 1);
        
        match dataset.get_column("name").unwrap() {
            ColumnDataArray::StringArray(vec) => {
                assert_eq!(vec.len(), 1);
                assert_eq!(vec[0], None);
            }
            _ => panic!("Wrong type"),
        }
    }

    #[test]
    fn test_invalid_operations2() {
        let mut dataset = ColDataSet::new("test".into());
        dataset.add_column("id".into(), ColumnType::I32).unwrap();
        dataset.add_column("name".into(), ColumnType::String).unwrap();

        // 模拟长度不一致的情况
        if let ColumnDataArray::NumberArray(vec) = dataset.columns.get_mut("id").unwrap() {
            vec.push(Some(1.into())); // 人为制造不一致
        }

        // 所有操作都应该失败
        let row = RowData::new();
        assert!(dataset.append_row(&row).is_err());
        assert!(dataset.insert_row(0, &row).is_err());
        assert!(dataset.remove_row(0).is_err());
        assert!(dataset.add_column("new_col".into(), ColumnType::Bool).is_err());
    }

    /// 嵌套数据集功能测试
    ///
    /// 验证子数据集的创建、添加、获取和移除功能。
    /// 测试层次化数据结构在列式存储中的实现。
    #[test]
    fn test_nested_dataset() {
        let mut parent_dataset = ColDataSet::new("parent".into());
        parent_dataset.add_column("id".into(), ColumnType::I32).unwrap();
        parent_dataset.add_column("name".into(), ColumnType::String).unwrap();

        // 创建子数据集
        let mut child_dataset = ColDataSet::new("child".into());
        child_dataset.add_column("item_id".into(), ColumnType::I32).unwrap();
        child_dataset.add_column("quantity".into(), ColumnType::I32).unwrap();

        // 添加子数据集的数据
        let mut child_row = RowData::new();
        child_row.insert("item_id".into(), CellValue::Number(1.into()));
        child_row.insert("quantity".into(), CellValue::Number(5.into()));
        child_dataset.append_row(&child_row).unwrap();

        // 创建父数据集的行，并包含子数据集
        let mut parent_row = RowData::new();
        parent_row.insert("id".into(), CellValue::Number(1.into()));
        parent_row.insert("name".into(), CellValue::String("Parent 1".into()));
        parent_row.add_child("items".into(), child_dataset);

        // 添加到父数据集
        parent_dataset.append_row(&parent_row).unwrap();

        // 验证数据
        assert_eq!(parent_dataset.row_count(), 1);
        
        // 获取并验证子数据集
        if let Some(removed_row) = parent_dataset.remove_row(0).ok() {
            if let Some(child_ds) = removed_row.get_child("items") {
                assert_eq!(child_ds.row_count(), 1);
                if let Some(ColumnDataArray::NumberArray(quantities)) = child_ds.get_column("quantity") {
                    assert_eq!(quantities[0].as_ref().unwrap().as_i64().unwrap(), 5);
                } else {
                    panic!("Wrong type for quantity column");
                }
            } else {
                panic!("Child dataset not found");
            }
        } else {
            panic!("Failed to remove row");
        }
    }

    #[test]
    fn test_dataset_id() {
        let dataset = ColDataSet::new("test_dataset".into());
        assert_eq!(dataset.dataset_id(), "test_dataset");
    }

    /// 子数据集添加功能测试
    ///
    /// 验证为行数据添加子数据集的基本功能。
    /// 测试子数据集映射的创建和管理。
    #[test]
    fn test_add_child() {
        let mut row = RowData::new();
        let child_dataset = ColDataSet::new("child".into());
        
        // 初始状态应该是 None
        assert!(row.get_child("child1").is_none());
        
        // 添加第一个子数据集
        row.add_child("child1".into(), child_dataset.clone());
        assert!(row.get_child("child1").is_some());
        
        // 添加第二个子数据集
        let child_dataset2 = ColDataSet::new("child2".into());
        row.add_child("child2".into(), child_dataset2);
        assert!(row.get_child("child2").is_some());
        
        // 验证数据集ID
        assert_eq!(row.get_child("child1").unwrap().dataset_id(), "child");
    }

    /// 子数据集操作测试
    ///
    /// 验证子数据集的添加、移除和生命周期管理。
    /// 测试子数据集映射的内存管理优化。
    #[test]
    fn test_child_dataset_operations() {
        let mut row = RowData::new();
        let child_dataset = ColDataSet::new("child".into());
        
        // 初始状态应该是 None
        assert!(row.get_child("child1").is_none());
        
        // 添加子数据集
        row.add_child("child1".into(), child_dataset.clone());
        assert!(row.get_child("child1").is_some());
        
        // 移除子数据集
        let removed = row.remove_child("child1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().dataset_id(), "child");
        
        // 验证移除后状态
        assert!(row.get_child("child1").is_none());
        // children 应该被设置为 None，因为 HashMap 为空
        assert!(matches!(row.children, None));
        
        // 测试移除不存在的子数据集
        assert!(row.remove_child("nonexistent").is_none());
    }

    /// 多子数据集管理测试
    ///
    /// 验证单行数据支持多个子数据集的功能。
    /// 测试子数据集映射在添加和移除操作中的内存优化行为。
    #[test]
    fn test_multiple_children() {
        let mut row = RowData::new();
        
        // 添加多个子数据集
        row.add_child("child1".into(), ColDataSet::new("child1".into()));
        row.add_child("child2".into(), ColDataSet::new("child2".into()));
        
        // 移除一个子数据集，children 应该仍然是 Some
        row.remove_child("child1");
        assert!(matches!(row.children, Some(_)));
        
        // 移除最后一个子数据集，children 应该变为 None
        row.remove_child("child2");
        assert!(matches!(row.children, None));
    }
}
