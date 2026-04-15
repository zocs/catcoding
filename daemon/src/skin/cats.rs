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
                display_name: "Cat Team".to_string(),
                motto: "Let AI collaborate like a cat team!".to_string(),
            },
            roles: vec![
                RoleSkin {
                    role: "pm".to_string(),
                    emoji: "🐱".to_string(),
                    name: "Siamese".to_string(),
                    description: "Smart, bossy, talkative → scheduling, reporting".to_string(),
                    mode: RoleMode::Resident,
                },
                RoleSkin {
                    role: "core_dev".to_string(),
                    emoji: "🐱".to_string(),
                    name: "British Shorthair".to_string(),
                    description: "Calm, reliable, efficient → core development".to_string(),
                    mode: RoleMode::OnDemand,
                },
                RoleSkin {
                    role: "frontend".to_string(),
                    emoji: "🐱".to_string(),
                    name: "Orange Tabby".to_string(),
                    description: "Warm, friendly → UI implementation".to_string(),
                    mode: RoleMode::OnDemand,
                },
                RoleSkin {
                    role: "backend".to_string(),
                    emoji: "🐱".to_string(),
                    name: "Maine Coon".to_string(),
                    description: "Big, powerful → backend, API".to_string(),
                    mode: RoleMode::OnDemand,
                },
                RoleSkin {
                    role: "reviewer".to_string(),
                    emoji: "🐱".to_string(),
                    name: "Bombay".to_string(),
                    description: "Mysterious, sharp → bug hunting".to_string(),
                    mode: RoleMode::Resident,
                },
                RoleSkin {
                    role: "tester".to_string(),
                    emoji: "🐱".to_string(),
                    name: "Abyssinian".to_string(),
                    description: "Curious, explorative → test writing".to_string(),
                    mode: RoleMode::OnDemand,
                },
                RoleSkin {
                    role: "deploy".to_string(),
                    emoji: "🐱".to_string(),
                    name: "Dragon Li".to_string(),
                    description: "Independent, adaptable → CI/CD, deployment".to_string(),
                    mode: RoleMode::OnDemand,
                },
                // 非猫角色
                RoleSkin {
                    role: "watchdog".to_string(),
                    emoji: "🦉".to_string(),
                    name: "Owl".to_string(),
                    description: "Nocturnal, alert, never sleeps = perfect watchdog".to_string(),
                    mode: RoleMode::Resident,
                },
                RoleSkin {
                    role: "tech_scout".to_string(),
                    emoji: "🦊".to_string(),
                    name: "Fox".to_string(),
                    description: "Smart, agile, discovery = tech scout".to_string(),
                    mode: RoleMode::Resident,
                },
                RoleSkin {
                    role: "mascot".to_string(),
                    emoji: "🐼".to_string(),
                    name: "Panda".to_string(),
                    description: "Does nothing, just cute and logo duty".to_string(),
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
