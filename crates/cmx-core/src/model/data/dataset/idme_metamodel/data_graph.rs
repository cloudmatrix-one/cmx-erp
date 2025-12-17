/// 华为iDME数据图谱模块
/// 
/// 提供业务数据的图谱化表示和追溯查询功能

use std::collections::{HashMap, VecDeque, HashSet};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// 数据节点 - 表示业务实体实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataNode {
    pub id: String,
    pub entity_instance_id: String,
    pub node_type: String, // 对应的实体类型
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl DataNode {
    pub fn new(
        entity_instance_id: String,
        node_type: String,
        properties: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            entity_instance_id,
            node_type,
            properties,
            created_at: Utc::now(),
        }
    }

    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    pub fn set_property(&mut self, key: String, value: serde_json::Value) {
        self.properties.insert(key, value);
    }
}

/// 数据边 - 表示业务实体间的关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataEdge {
    pub id: String,
    pub source_node_id: String,
    pub target_node_id: String,
    pub relation_instance_id: String,
    pub edge_type: String, // 关系类型
    pub properties: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl DataEdge {
    pub fn new(
        source_node_id: String,
        target_node_id: String,
        relation_instance_id: String,
        edge_type: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source_node_id,
            target_node_id,
            relation_instance_id,
            edge_type,
            properties: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    pub fn with_properties(mut self, properties: HashMap<String, serde_json::Value>) -> Self {
        self.properties = properties;
        self
    }
}

/// 追溯路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracePath {
    pub source_node_id: String,
    pub target_node_id: String,
    pub nodes: Vec<String>, // 路径上的节点ID序列
    pub edges: Vec<String>, // 路径上的边ID序列
    pub path_length: usize,
    pub total_weight: f64,
    pub created_at: DateTime<Utc>,
}

impl TracePath {
    pub fn new(
        source_node_id: String,
        target_node_id: String,
        nodes: Vec<String>,
        edges: Vec<String>,
    ) -> Self {
        let path_length = nodes.len();
        Self {
            source_node_id,
            target_node_id,
            nodes,
            edges,
            path_length,
            total_weight: 0.0,
            created_at: Utc::now(),
        }
    }

    pub fn with_weight(mut self, weight: f64) -> Self {
        self.total_weight = weight;
        self
    }
}

/// 数据图谱
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGraph {
    pub nodes: HashMap<String, DataNode>,
    pub edges: HashMap<String, DataEdge>,
    pub node_index: HashMap<String, Vec<String>>, // node_type -> node_ids
    pub edge_index: HashMap<String, Vec<String>>, // edge_type -> edge_ids
    pub adjacency_list: HashMap<String, Vec<String>>, // node_id -> connected_edge_ids
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DataGraph {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            node_index: HashMap::new(),
            edge_index: HashMap::new(),
            adjacency_list: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 添加节点
    pub fn add_node(&mut self, node: DataNode) {
        // 添加到类型索引
        self.node_index
            .entry(node.node_type.clone())
            .or_insert_with(Vec::new)
            .push(node.id.clone());

        // 初始化邻接表
        self.adjacency_list.insert(node.id.clone(), Vec::new());

        self.nodes.insert(node.id.clone(), node);
        self.updated_at = Utc::now();
    }

    /// 添加边
    pub fn add_edge(&mut self, edge: DataEdge) {
        // 验证源节点和目标节点存在
        if !self.nodes.contains_key(&edge.source_node_id) || !self.nodes.contains_key(&edge.target_node_id) {
            return; // 静默失败，实际应用中可以返回错误
        }

        // 添加到类型索引
        self.edge_index
            .entry(edge.edge_type.clone())
            .or_insert_with(Vec::new)
            .push(edge.id.clone());

        // 更新邻接表
        self.adjacency_list
            .entry(edge.source_node_id.clone())
            .or_insert_with(Vec::new)
            .push(edge.id.clone());

        self.adjacency_list
            .entry(edge.target_node_id.clone())
            .or_insert_with(Vec::new)
            .push(edge.id.clone());

        self.edges.insert(edge.id.clone(), edge);
        self.updated_at = Utc::now();
    }

    /// 获取节点的邻居节点
    pub fn get_neighbors(&self, node_id: &str) -> Vec<&DataNode> {
        let mut neighbors = Vec::new();
        
        if let Some(edge_ids) = self.adjacency_list.get(node_id) {
            for edge_id in edge_ids {
                if let Some(edge) = self.edges.get(edge_id) {
                    let neighbor_id = if edge.source_node_id == node_id {
                        &edge.target_node_id
                    } else {
                        &edge.source_node_id
                    };
                    
                    if let Some(neighbor) = self.nodes.get(neighbor_id) {
                        neighbors.push(neighbor);
                    }
                }
            }
        }
        
        neighbors
    }

    /// 根据类型获取节点
    pub fn get_nodes_by_type(&self, node_type: &str) -> Vec<&DataNode> {
        if let Some(node_ids) = self.node_index.get(node_type) {
            node_ids.iter()
                .filter_map(|id| self.nodes.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 根据类型获取边
    pub fn get_edges_by_type(&self, edge_type: &str) -> Vec<&DataEdge> {
        if let Some(edge_ids) = self.edge_index.get(edge_type) {
            edge_ids.iter()
                .filter_map(|id| self.edges.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 删除节点
    pub fn remove_node(&mut self, node_id: &str) -> Option<DataNode> {
        if let Some(node) = self.nodes.remove(node_id) {
            // 从类型索引中移除
            if let Some(node_ids) = self.node_index.get_mut(&node.node_type) {
                node_ids.retain(|id| id != node_id);
            }

            // 删除相关的边
            let related_edges: Vec<String> = self.edges.values()
                .filter(|edge| edge.source_node_id == node_id || edge.target_node_id == node_id)
                .map(|edge| edge.id.clone())
                .collect();

            for edge_id in related_edges {
                self.remove_edge(&edge_id);
            }

            // 从邻接表中移除
            self.adjacency_list.remove(node_id);

            self.updated_at = Utc::now();
            Some(node)
        } else {
            None
        }
    }

    /// 删除边
    pub fn remove_edge(&mut self, edge_id: &str) -> Option<DataEdge> {
        if let Some(edge) = self.edges.remove(edge_id) {
            // 从类型索引中移除
            if let Some(edge_ids) = self.edge_index.get_mut(&edge.edge_type) {
                edge_ids.retain(|id| id != edge_id);
            }

            // 从邻接表中移除
            if let Some(source_edges) = self.adjacency_list.get_mut(&edge.source_node_id) {
                source_edges.retain(|id| id != edge_id);
            }
            if let Some(target_edges) = self.adjacency_list.get_mut(&edge.target_node_id) {
                target_edges.retain(|id| id != edge_id);
            }

            self.updated_at = Utc::now();
            Some(edge)
        } else {
            None
        }
    }

    /// 获取图谱统计信息
    pub fn get_statistics(&self) -> GraphStatistics {
        let mut node_type_counts = HashMap::new();
        for node in self.nodes.values() {
            *node_type_counts.entry(node.node_type.clone()).or_insert(0) += 1;
        }

        let mut edge_type_counts = HashMap::new();
        for edge in self.edges.values() {
            *edge_type_counts.entry(edge.edge_type.clone()).or_insert(0) += 1;
        }

        GraphStatistics {
            total_nodes: self.nodes.len(),
            total_edges: self.edges.len(),
            node_type_counts,
            edge_type_counts,
            created_at: Utc::now(),
        }
    }
}

impl Default for DataGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// 图谱统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStatistics {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub node_type_counts: HashMap<String, usize>,
    pub edge_type_counts: HashMap<String, usize>,
    pub created_at: DateTime<Utc>,
}

/// 图谱查询引擎
#[derive(Debug, Clone)]
pub struct GraphQueryEngine {
    pub graph: DataGraph,
}

impl GraphQueryEngine {
    pub fn new(graph: DataGraph) -> Self {
        Self { graph }
    }

    /// 使用BFS算法进行路径追溯
    pub fn trace_path(
        &self,
        source_node_id: &str,
        target_node_id: &str,
        max_depth: usize,
    ) -> Option<TracePath> {
        if !self.graph.nodes.contains_key(source_node_id) || !self.graph.nodes.contains_key(target_node_id) {
            return None;
        }

        if source_node_id == target_node_id {
            return Some(TracePath::new(
                source_node_id.to_string(),
                target_node_id.to_string(),
                vec![source_node_id.to_string()],
                vec![],
            ));
        }

        let mut queue = VecDeque::new();
        let mut visited = HashSet::new();
        let mut parent_map: HashMap<String, (String, String)> = HashMap::new(); // node_id -> (parent_node_id, edge_id)

        queue.push_back((source_node_id.to_string(), 0));
        visited.insert(source_node_id.to_string());

        while let Some((current_node_id, depth)) = queue.pop_front() {
            if depth >= max_depth {
                continue;
            }

            if let Some(edge_ids) = self.graph.adjacency_list.get(&current_node_id) {
                for edge_id in edge_ids {
                    if let Some(edge) = self.graph.edges.get(edge_id) {
                        let next_node_id = if edge.source_node_id == current_node_id {
                            &edge.target_node_id
                        } else {
                            &edge.source_node_id
                        };

                        if !visited.contains(next_node_id) {
                            visited.insert(next_node_id.clone());
                            parent_map.insert(next_node_id.clone(), (current_node_id.clone(), edge_id.clone()));
                            queue.push_back((next_node_id.clone(), depth + 1));

                            if next_node_id == target_node_id {
                                // 找到目标节点，重建路径
                                return Some(self.reconstruct_path(
                                    source_node_id,
                                    target_node_id,
                                    &parent_map,
                                ));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// 重建路径
    fn reconstruct_path(
        &self,
        source_node_id: &str,
        target_node_id: &str,
        parent_map: &HashMap<String, (String, String)>,
    ) -> TracePath {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut current = target_node_id.to_string();

        // 从目标节点向源节点回溯
        nodes.push(current.clone());
        while let Some((parent_node_id, edge_id)) = parent_map.get(&current) {
            edges.push(edge_id.clone());
            nodes.push(parent_node_id.clone());
            current = parent_node_id.clone();

            if current == source_node_id {
                break;
            }
        }

        // 反转路径（因为是从目标向源回溯的）
        nodes.reverse();
        edges.reverse();

        TracePath::new(
            source_node_id.to_string(),
            target_node_id.to_string(),
            nodes,
            edges,
        )
    }

    /// 查找所有路径（限制最大深度）
    pub fn find_all_paths(
        &self,
        source_node_id: &str,
        target_node_id: &str,
        max_depth: usize,
    ) -> Vec<TracePath> {
        let mut paths = Vec::new();
        let mut current_path = Vec::new();
        let mut current_edges = Vec::new();
        let mut visited = HashSet::new();

        self.dfs_find_paths(
            source_node_id,
            target_node_id,
            &mut current_path,
            &mut current_edges,
            &mut visited,
            &mut paths,
            max_depth,
            0,
        );

        paths
    }

    /// 深度优先搜索查找所有路径
    fn dfs_find_paths(
        &self,
        current_node_id: &str,
        target_node_id: &str,
        current_path: &mut Vec<String>,
        current_edges: &mut Vec<String>,
        visited: &mut HashSet<String>,
        paths: &mut Vec<TracePath>,
        max_depth: usize,
        depth: usize,
    ) {
        if depth > max_depth {
            return;
        }

        current_path.push(current_node_id.to_string());
        visited.insert(current_node_id.to_string());

        if current_node_id == target_node_id {
            // 找到目标节点，保存路径
            paths.push(TracePath::new(
                current_path[0].clone(),
                target_node_id.to_string(),
                current_path.clone(),
                current_edges.clone(),
            ));
        } else if let Some(edge_ids) = self.graph.adjacency_list.get(current_node_id) {
            for edge_id in edge_ids {
                if let Some(edge) = self.graph.edges.get(edge_id) {
                    let next_node_id = if edge.source_node_id == current_node_id {
                        &edge.target_node_id
                    } else {
                        &edge.source_node_id
                    };

                    if !visited.contains(next_node_id) {
                        current_edges.push(edge_id.clone());
                        self.dfs_find_paths(
                            next_node_id,
                            target_node_id,
                            current_path,
                            current_edges,
                            visited,
                            paths,
                            max_depth,
                            depth + 1,
                        );
                        current_edges.pop();
                    }
                }
            }
        }

        current_path.pop();
        visited.remove(current_node_id);
    }

    /// 根据节点属性查询
    pub fn query_nodes_by_property(
        &self,
        property_key: &str,
        property_value: &serde_json::Value,
    ) -> Vec<&DataNode> {
        self.graph.nodes.values()
            .filter(|node| {
                node.get_property(property_key)
                    .map_or(false, |v| v == property_value)
            })
            .collect()
    }

    /// 根据节点类型和属性查询
    pub fn query_nodes_by_type_and_property(
        &self,
        node_type: &str,
        property_key: &str,
        property_value: &serde_json::Value,
    ) -> Vec<&DataNode> {
        self.graph.get_nodes_by_type(node_type)
            .into_iter()
            .filter(|node| {
                node.get_property(property_key)
                    .map_or(false, |v| v == property_value)
            })
            .collect()
    }

    /// 获取节点的度数（连接的边数）
    pub fn get_node_degree(&self, node_id: &str) -> usize {
        self.graph.adjacency_list.get(node_id)
            .map_or(0, |edges| edges.len())
    }

    /// 获取图的连通分量
    pub fn get_connected_components(&self) -> Vec<Vec<String>> {
        let mut visited = HashSet::new();
        let mut components = Vec::new();

        for node_id in self.graph.nodes.keys() {
            if !visited.contains(node_id) {
                let mut component = Vec::new();
                self.dfs_component(node_id, &mut visited, &mut component);
                components.push(component);
            }
        }

        components
    }

    /// 深度优先搜索连通分量
    fn dfs_component(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        component: &mut Vec<String>,
    ) {
        visited.insert(node_id.to_string());
        component.push(node_id.to_string());

        if let Some(edge_ids) = self.graph.adjacency_list.get(node_id) {
            for edge_id in edge_ids {
                if let Some(edge) = self.graph.edges.get(edge_id) {
                    let neighbor_id = if edge.source_node_id == node_id {
                        &edge.target_node_id
                    } else {
                        &edge.source_node_id
                    };

                    if !visited.contains(neighbor_id) {
                        self.dfs_component(neighbor_id, visited, component);
                    }
                }
            }
        }
    }
}
