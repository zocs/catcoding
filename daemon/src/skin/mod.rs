pub mod cats;
// pub mod dogs;

use serde::{Deserialize, Serialize};

/// 角色皮肤信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinInfo {
    pub name: String,
    pub display_name: String,
    pub motto: String,
}

/// 角色定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSkin {
    pub role: String,
    pub emoji: String,
    pub name: String,
    pub description: String,
    pub mode: RoleMode,
}

/// 角色模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleMode {
    /// 常驻
    Resident,
    /// 按需派生
    OnDemand,
    /// 装饰
    Decorative,
}

/// 皮肤系统 — 定义 Agent 形象
///
/// 支持多套皮肤：
/// - cats: 默认猫系（当前）
/// - dogs: 狗系（未来扩展）
/// - pandas: 熊猫系（极简风）
pub trait Skin {
    fn info(&self) -> &SkinInfo;
    fn roles(&self) -> &[RoleSkin];
    fn get_role(&self, role_name: &str) -> Option<&RoleSkin>;
}
