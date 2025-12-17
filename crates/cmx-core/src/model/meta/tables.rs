use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize,EnumString, Display)]
pub enum SYS_TABLE_NAMES {
    SYS_MODEL,          // 模型表
    SYS_MDL_CTN,        // 模型内容表
    SYS_MDL_VAL,        // 模型值表
    SYS_RLGL,           // 关系管理表
    SYS_DICTS,          // 数据字典表
    SYS_DCT_CST,        // 数据字典自定义表
    SYS_KEYS,           // 主键管理表
    SYS_INDEXS,         // 索引表
    SYS_OBJCOLS,        // 对象列表
    SYS_OBJECTS,        // 对象表
    SYS_OBJ_VAL,        // 对象值表
    SYS_FACTS,          // 事实表
    SYS_OPLOG,          // 操作日志表
    BSCONF,             // 基础配置表
}

// pub enum TableId {
//     SYS_OBJECTS,        // 系统对象表
//     SYS_OBJCOLS,        // 系统对象列表
//     SYS_DICTS,          // 系统字典表
//     SYS_DICTCOLS,       // 系统字典列表
//     SYS_USERS,          // 系统用户表
//     SYS_ROLES,          // 系统角色表
//     SYS_MENUS,          // 系统菜单表
//     SYS_PERMISSIONS,    // 系统权限表
//     SYS_LOGS,           // 系统日志表
//     SYS_SETTINGS,       // 系统设置表
//     SYS_UNITS,          // 系统单位表
//     SYS_LANGS,          // 系统语言表
//     SYS_PARAMS,         // 系统参数表
//     SYS_CODES,          // 系统代码表
//     SYS_SEQS,          // 系统序列表
//     SYS_MSGS,          // 系统消息表
//     SYS_TASKS,         // 系统任务表
//     SYS_TASK_LOGS,     // 系统任务日志表
//     SYS_FILES,         // 系统文件表
//     SYS_FILE_DIRS,     // 系统文件目录表
//     SYS_TEMPLATES,     // 系统模板表
//     SYS_WORKFLOWS,     // 系统工作流表
//     SYS_WORKFLOW_NODES, // 系统工作流节点表
//     SYS_WORKFLOW_LOGS, // 系统工作流日志表
//     SYS_APIS,         // 系统API表
//     SYS_API_LOGS,     // 系统API日志表
//     SYS_CONFIGS,      // 系统配置表
//     SYS_VERSIONS,     // 系统版本表
//     SYS_BACKUPS,      // 系统备份表
//     SYS_ATTACHMENTS,  // 系统附件表
//     SYS_NOTICES,      // 系统通知表
//     SYS_NOTICE_USERS, // 系统通知用户表
//     SYS_REGIONS,      // 系统区域表
//     SYS_ORGS,         // 系统组织表
//     SYS_POSITIONS,    // 系统职位表
//     SYS_USER_ROLES,   // 系统用户角色表
//     SYS_ROLE_MENUS,   // 系统角色菜单表
//     SYS_ROLE_PERMISSIONS, // 系统角色权限表
//     SYS_USER_POSITIONS,   // 系统用户职位表
//     SYS_USER_ORGS,       // 系统用户组织表
// }

impl SYS_TABLE_NAMES {

    pub fn from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }
    
    pub fn get_table_name(&self) -> &'static str {
        match self {
            SYS_TABLE_NAMES::SYS_MODEL => "SYS_MODEL",
            SYS_TABLE_NAMES::SYS_MDL_CTN => "SYS_MDL_CTN",
            SYS_TABLE_NAMES::SYS_MDL_VAL => "SYS_MDL_VAL",
            SYS_TABLE_NAMES::SYS_RLGL => "SYS_RLGL",
            SYS_TABLE_NAMES::SYS_DICTS => "SYS_DICTS",
            SYS_TABLE_NAMES::SYS_DCT_CST => "SYS_DCT_CST",
            SYS_TABLE_NAMES::SYS_KEYS => "SYS_KEYS",
            SYS_TABLE_NAMES::SYS_INDEXS => "SYS_INDEXS",
            SYS_TABLE_NAMES::SYS_OBJCOLS => "SYS_OBJCOLS",
            SYS_TABLE_NAMES::SYS_OBJECTS => "SYS_OBJECTS",
            SYS_TABLE_NAMES::SYS_OBJ_VAL => "SYS_OBJ_VAL",
            SYS_TABLE_NAMES::SYS_FACTS => "SYS_FACTS",
            SYS_TABLE_NAMES::SYS_OPLOG => "SYS_OPLOG",
            SYS_TABLE_NAMES::BSCONF => "BSCONF",
        }
    }
}