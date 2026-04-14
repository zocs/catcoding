use super::{RoleMode, RoleSkin, Skin, SkinInfo};

/// 猫系皮肤 — 默认主题
pub struct CatSkin {
    info: SkinInfo,
    roles: Vec<RoleSkin>,
}

impl CatSkin {
    pub fn new() -> Self {
        Self {
            info: SkinInfo {
                name: "cats".to_string(),
                display_name: "猫咪团队".to_string(),
                motto: "🐱 让 AI 像猫咪团队一样协作做菜！".to_string(),
            },
            roles: vec![
                RoleSkin {
                    role: "pm".to_string(),
                    emoji: "🐱".to_string(),
                    name: "暹罗猫".to_string(),
                    description: "聪明、爱指挥、话多 → 全局观、调度、汇报".to_string(),
                    mode: RoleMode::Resident,
                },
                RoleSkin {
                    role: "core_dev".to_string(),
                    emoji: "🐱".to_string(),
                    name: "英短蓝猫".to_string(),
                    description: "沉稳、可靠、高效 → 核心功能开发".to_string(),
                    mode: RoleMode::OnDemand,
                },
                RoleSkin {
                    role: "frontend".to_string(),
                    emoji: "🐱".to_string(),
                    name: "橘猫".to_string(),
                    description: "温暖、亲和力强 → 用户界面实现".to_string(),
                    mode: RoleMode::OnDemand,
                },
                RoleSkin {
                    role: "backend".to_string(),
                    emoji: "🐱".to_string(),
                    name: "缅因猫".to_string(),
                    description: "体型大、力量强 → 后端、API".to_string(),
                    mode: RoleMode::OnDemand,
                },
                RoleSkin {
                    role: "reviewer".to_string(),
                    emoji: "🐱".to_string(),
                    name: "玄猫".to_string(),
                    description: "神秘、敏锐 → 找 bug（抓老鼠）".to_string(),
                    mode: RoleMode::Resident,
                },
                RoleSkin {
                    role: "tester".to_string(),
                    emoji: "🐱".to_string(),
                    name: "阿比西尼亚猫".to_string(),
                    description: "好奇心强、爱探索 → 测试用例编写".to_string(),
                    mode: RoleMode::OnDemand,
                },
                RoleSkin {
                    role: "deploy".to_string(),
                    emoji: "🐱".to_string(),
                    name: "狸花猫".to_string(),
                    description: "独立、适应力强 → CI/CD、部署".to_string(),
                    mode: RoleMode::OnDemand,
                },
                // 非猫角色
                RoleSkin {
                    role: "watchdog".to_string(),
                    emoji: "🦉".to_string(),
                    name: "猫头鹰".to_string(),
                    description: "夜行、警觉、永远不睡觉 = 完美的守护进程".to_string(),
                    mode: RoleMode::Resident,
                },
                RoleSkin {
                    role: "tech_scout".to_string(),
                    emoji: "🦊".to_string(),
                    name: "狐狸".to_string(),
                    description: "聪明、敏捷、善于发现 = 技术侦察兵".to_string(),
                    mode: RoleMode::Resident,
                },
                RoleSkin {
                    role: "mascot".to_string(),
                    emoji: "🐼".to_string(),
                    name: "大熊猫".to_string(),
                    description: "不干活，只负责可爱和 logo".to_string(),
                    mode: RoleMode::Decorative,
                },
            ],
        }
    }
}

impl Skin for CatSkin {
    fn info(&self) -> &SkinInfo {
        &self.info
    }

    fn roles(&self) -> &[RoleSkin] {
        &self.roles
    }

    fn get_role(&self, role_name: &str) -> Option<&RoleSkin> {
        self.roles.iter().find(|r| r.role == role_name)
    }
}
