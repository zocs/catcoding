//! Agent XP / 等级系统 — 基础类型与纯函数
//!
//! 计划书 §4.6 定义的 5 级系统。当前本模块只包含类型、等级计算和 XP 规则，
//! 尚未接入 scheduler/watchdog 的事件回调——接线在下一次会话完成。
//!
//! 设计目标：让 `Scheduler` 完成任务时调用 `XpEngine::apply(event)`，引擎根据
//! 事件查规则、更新 `AgentInfo.level/xp`、写 `xp_log` 表、并广播升级事件到
//! Dashboard。接线点以及完整引擎留到下一会话。

use serde::{Deserialize, Serialize};

/// XP 门槛 — 下标 i 表示升到 i+1 级所需累计 XP。
/// Lv1=0, Lv2=50, Lv3=200, Lv4=500, Lv5=1000.
pub const XP_THRESHOLDS: [u32; 5] = [0, 50, 200, 500, 1000];

/// 最大等级
pub const MAX_LEVEL: u32 = 5;

/// 根据累计 XP 计算等级。
pub fn level_for_xp(xp: u32) -> u32 {
    let mut level: u32 = 1;
    for (i, &threshold) in XP_THRESHOLDS.iter().enumerate() {
        if xp >= threshold {
            level = (i as u32) + 1;
        }
    }
    level.min(MAX_LEVEL)
}

/// 下一级需要的 XP；若已到顶级则返回当前等级门槛。
pub fn xp_for_next_level(level: u32) -> u32 {
    let idx = (level as usize).min(XP_THRESHOLDS.len() - 1);
    XP_THRESHOLDS[idx]
}

/// 当前等级起点 XP。
pub fn xp_for_current_level(level: u32) -> u32 {
    if level == 0 {
        return 0;
    }
    let idx = ((level - 1) as usize).min(XP_THRESHOLDS.len() - 1);
    XP_THRESHOLDS[idx]
}

/// XP 触发事件 — 与计划书 §4.6 xp_rules 对应。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum XpEvent {
    TaskCompleted,
    ReviewPassedFirst,
    ReviewPassedAfterRetry,
    BugFixed,
    ComplexTaskBonus,
    TaskFailed,
    Timeout,
    StreakBonus { streak: u32 },
}

impl XpEvent {
    /// 事件对应的 XP 增量（可负）。
    pub fn delta(&self) -> i32 {
        match self {
            XpEvent::TaskCompleted => 10,
            XpEvent::ReviewPassedFirst => 15,
            XpEvent::ReviewPassedAfterRetry => 5,
            XpEvent::BugFixed => 8,
            XpEvent::ComplexTaskBonus => 10,
            XpEvent::TaskFailed => -5,
            XpEvent::Timeout => -3,
            XpEvent::StreakBonus { streak } => match *streak {
                5 => 20,
                10 => 50,
                _ => 0,
            },
        }
    }

    /// 用于 xp_log.reason 字段。
    pub fn reason(&self) -> &'static str {
        match self {
            XpEvent::TaskCompleted => "task_completed",
            XpEvent::ReviewPassedFirst => "review_passed_first",
            XpEvent::ReviewPassedAfterRetry => "review_passed_after_retry",
            XpEvent::BugFixed => "bug_fixed",
            XpEvent::ComplexTaskBonus => "complex_task_bonus",
            XpEvent::TaskFailed => "task_failed",
            XpEvent::Timeout => "timeout",
            XpEvent::StreakBonus { .. } => "consecutive_success_bonus",
        }
    }
}

/// 应用一个 XP 事件的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XpOutcome {
    pub old_xp: u32,
    pub new_xp: u32,
    pub old_level: u32,
    pub new_level: u32,
    pub delta: i32,
    pub leveled_up: bool,
}

/// 把一个事件应用到 (xp, level) 上，返回新状态。XP 不会低于 0。
pub fn apply_event(current_xp: u32, current_level: u32, event: &XpEvent) -> XpOutcome {
    let delta = event.delta();
    let new_xp_signed = current_xp as i64 + delta as i64;
    let new_xp = new_xp_signed.max(0) as u32;
    let new_level = level_for_xp(new_xp);
    XpOutcome {
        old_xp: current_xp,
        new_xp,
        old_level: current_level,
        new_level,
        delta,
        leveled_up: new_level > current_level,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_for_xp_boundaries() {
        assert_eq!(level_for_xp(0), 1);
        assert_eq!(level_for_xp(49), 1);
        assert_eq!(level_for_xp(50), 2);
        assert_eq!(level_for_xp(199), 2);
        assert_eq!(level_for_xp(200), 3);
        assert_eq!(level_for_xp(499), 3);
        assert_eq!(level_for_xp(500), 4);
        assert_eq!(level_for_xp(999), 4);
        assert_eq!(level_for_xp(1000), 5);
        assert_eq!(level_for_xp(u32::MAX), 5);
    }

    #[test]
    fn test_xp_for_next_level() {
        assert_eq!(xp_for_next_level(1), 50);
        assert_eq!(xp_for_next_level(2), 200);
        assert_eq!(xp_for_next_level(3), 500);
        assert_eq!(xp_for_next_level(4), 1000);
        // 顶级时，返回最后一个门槛
        assert_eq!(xp_for_next_level(5), 1000);
    }

    #[test]
    fn test_xp_for_current_level() {
        assert_eq!(xp_for_current_level(1), 0);
        assert_eq!(xp_for_current_level(2), 50);
        assert_eq!(xp_for_current_level(3), 200);
        assert_eq!(xp_for_current_level(5), 1000);
    }

    #[test]
    fn test_event_delta_table() {
        assert_eq!(XpEvent::TaskCompleted.delta(), 10);
        assert_eq!(XpEvent::ReviewPassedFirst.delta(), 15);
        assert_eq!(XpEvent::ReviewPassedAfterRetry.delta(), 5);
        assert_eq!(XpEvent::BugFixed.delta(), 8);
        assert_eq!(XpEvent::ComplexTaskBonus.delta(), 10);
        assert_eq!(XpEvent::TaskFailed.delta(), -5);
        assert_eq!(XpEvent::Timeout.delta(), -3);
        assert_eq!(XpEvent::StreakBonus { streak: 5 }.delta(), 20);
        assert_eq!(XpEvent::StreakBonus { streak: 10 }.delta(), 50);
        assert_eq!(XpEvent::StreakBonus { streak: 7 }.delta(), 0);
    }

    #[test]
    fn test_apply_event_level_up() {
        // 40 + 10 (TaskCompleted) = 50 → level 1 → 2
        let outcome = apply_event(40, 1, &XpEvent::TaskCompleted);
        assert_eq!(outcome.old_xp, 40);
        assert_eq!(outcome.new_xp, 50);
        assert_eq!(outcome.old_level, 1);
        assert_eq!(outcome.new_level, 2);
        assert!(outcome.leveled_up);
        assert_eq!(outcome.delta, 10);
    }

    #[test]
    fn test_apply_event_xp_floor_at_zero() {
        let outcome = apply_event(2, 1, &XpEvent::TaskFailed);
        assert_eq!(outcome.new_xp, 0);
        assert_eq!(outcome.new_level, 1);
        assert!(!outcome.leveled_up);
    }

    #[test]
    fn test_apply_event_stays_at_max() {
        let outcome = apply_event(1000, 5, &XpEvent::TaskCompleted);
        assert_eq!(outcome.new_xp, 1010);
        assert_eq!(outcome.new_level, 5); // already maxed
        assert!(!outcome.leveled_up);
    }
}
