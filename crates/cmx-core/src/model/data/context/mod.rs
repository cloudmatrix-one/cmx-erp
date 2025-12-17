use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::cell::CellValue;


/// Context结构体用于管理键值对形式的上下文数据
#[derive(Clone, Debug)]
pub struct SVRContext {
    data: Arc<RwLock<HashMap<String, CellValue>>>,
}

impl SVRContext {
    /// 创建一个新的Context实例
    pub fn new() -> Self {
        SVRContext {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 设置键值对
    pub fn set<K: Into<String>>(&self, key: K, value: CellValue) {
        let mut data = self.data.write().unwrap();
        data.insert(key.into(), value);
    }

    /// 获取指定键的值
    pub fn get<K: AsRef<str>>(&self, key: K) -> Option<CellValue> {
        let data = self.data.read().unwrap();
        data.get(key.as_ref()).cloned()
    }

    /// 删除指定键的值
    pub fn remove<K: AsRef<str>>(&self, key: K) -> Option<CellValue> {
        let mut data = self.data.write().unwrap();
        data.remove(key.as_ref())
    }

    /// 检查是否包含指定键
    pub fn contains_key<K: AsRef<str>>(&self, key: K) -> bool {
        let data = self.data.read().unwrap();
        data.contains_key(key.as_ref())
    }

    /// 清空所有数据
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        data.clear();
    }
}

impl Default for SVRContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_operations() {
        let ctx = SVRContext::new();
        
        // 测试设置和获取值
        ctx.set("name", CellValue::String("test".to_string()));
        ctx.set("age", CellValue::Number(25.into()));
        
        assert!(ctx.contains_key("name"));
        assert!(ctx.contains_key("age"));
        
        if let Some(CellValue::String(name)) = ctx.get("name") {
            assert_eq!(name.as_str(), "test");
        }
        
        if let Some(CellValue::Number(age)) = ctx.get("age") {
            assert_eq!(age.as_i64().unwrap(), 25);
        }
        
        // 测试删除值
        ctx.remove("name");
        assert!(!ctx.contains_key("name"));
        
        // 测试清空
        ctx.clear();
        assert!(!ctx.contains_key("age"));
    }
}
