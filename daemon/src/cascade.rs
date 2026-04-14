use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::{Task, TaskStatus};

/// 依赖图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// task_id → 依赖它的任务列表
    dependents: HashMap<String, Vec<String>>,
    /// task_id → 它依赖的任务列表
    dependencies: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependents: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// 添加依赖关系
    pub fn add_dependency(&mut self, task_id: &str, depends_on: &str) {
        self.dependents
            .entry(depends_on.to_string())
            .or_default()
            .push(task_id.to_string());

        self.dependencies
            .entry(task_id.to_string())
            .or_default()
            .push(depends_on.to_string());
    }

    /// 获取直接依赖
    pub fn get_dependencies(&self, task_id: &str) -> Vec<String> {
        self.dependencies.get(task_id).cloned().unwrap_or_default()
    }

    /// 获取直接依赖者
    pub fn get_dependents(&self, task_id: &str) -> Vec<String> {
        self.dependents.get(task_id).cloned().unwrap_or_default()
    }

    /// 获取所有下游依赖者（BFS）
    pub fn get_all_downstream(&self, task_id: &str) -> Vec<String> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back(task_id.to_string());
        visited.insert(task_id.to_string());

        while let Some(current) = queue.pop_front() {
            for dependent in self.get_dependents(&current) {
                if !visited.contains(&dependent) {
                    visited.insert(dependent.clone());
                    queue.push_back(dependent.clone());
                    result.push(dependent);
                }
            }
        }

        result
    }

    /// 检查是否存在循环依赖
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for task_id in self.dependencies.keys() {
            if self._has_cycle_dfs(task_id, &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    fn _has_cycle_dfs(
        &self,
        task_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(task_id.to_string());
        rec_stack.insert(task_id.to_string());

        for dependent in self.get_dependents(task_id) {
            if !visited.contains(&dependent) {
                if self._has_cycle_dfs(&dependent, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&dependent) {
                return true;
            }
        }

        rec_stack.remove(task_id);
        false
    }

    /// 拓扑排序
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        // 计算入度
        for (task_id, deps) in &self.dependencies {
            in_degree.entry(task_id.clone()).or_insert(deps.len());
        }

        // 找到入度为 0 的节点
        for task_id in self.dependencies.keys() {
            if in_degree.get(task_id) == Some(&0) {
                queue.push_back(task_id.clone());
            }
        }

        // BFS
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            for dependent in self.get_dependents(&current) {
                if let Some(degree) = in_degree.get_mut(&dependent) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }

        if result.len() != self.dependencies.len() {
            return Err(anyhow::anyhow!("存在循环依赖"));
        }

        Ok(result)
    }
}

/// 级联处理器
///
/// 处理任务失败/阻塞时的级联影响
pub struct CascadeHandler {
    graph: Arc<RwLock<DependencyGraph>>,
}

/// 级联影响
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CascadeImpact {
    pub failed_task: String,
    pub blocked_tasks: Vec<String>,
    pub can_continue: Vec<String>,
    pub requires_replan: bool,
}

impl CascadeHandler {
    pub fn new() -> Self {
        Self {
            graph: Arc::new(RwLock::new(DependencyGraph::new())),
        }
    }

    /// 从任务列表构建依赖图
    pub async fn build_from_tasks(&self, tasks: &[Task]) {
        let mut graph = self.graph.write().await;
        *graph = DependencyGraph::new();

        for task in tasks {
            for dep in &task.depends_on {
                graph.add_dependency(&task.id, dep);
            }
        }

        // 检查循环依赖
        if graph.has_cycle() {
            tracing::error!("⚠️ 检测到循环依赖！");
        }
    }

    /// 分析任务失败的级联影响
    pub async fn analyze_impact(&self, task_id: &str, tasks: &[Task]) -> CascadeImpact {
        let graph = self.graph.read().await;

        // 获取所有下游依赖者
        let downstream = graph.get_all_downstream(task_id);

        // 分类：哪些可以继续，哪些被阻塞
        let mut blocked = Vec::new();
        let mut can_continue = Vec::new();

        for dep_id in &downstream {
            if let Some(task) = tasks.iter().find(|t| &t.id == dep_id) {
                match task.status {
                    TaskStatus::Done => {
                        can_continue.push(dep_id.clone());
                    }
                    _ => {
                        blocked.push(dep_id.clone());
                    }
                }
            }
        }

        let requires_replan = blocked.len() > 3 || !blocked.is_empty();

        CascadeImpact {
            failed_task: task_id.to_string(),
            blocked_tasks: blocked,
            can_continue,
            requires_replan,
        }
    }

    /// 处理任务状态变更
    pub async fn handle_status_change(
        &self,
        task_id: &str,
        new_status: &TaskStatus,
        tasks: &mut Vec<Task>,
    ) -> Result<Vec<String>> {
        let mut affected = Vec::new();
        let graph = self.graph.read().await;

        match new_status {
            TaskStatus::Done => {
                // 任务完成，检查依赖它的任务是否可以变为 Ready
                let dependents = graph.get_dependents(task_id);

                // 先收集需要更新的任务
                let mut to_update = Vec::new();
                for dep_id in &dependents {
                    let deps = graph.get_dependencies(dep_id);
                    let all_done = deps.iter().all(|d| {
                        tasks
                            .iter()
                            .any(|t| &t.id == d && t.status == TaskStatus::Done)
                    });

                    if all_done {
                        if let Some(task) = tasks.iter().find(|t| t.id == *dep_id) {
                            if task.status == TaskStatus::Blocked
                                || task.status == TaskStatus::Pending
                            {
                                to_update.push(dep_id.clone());
                            }
                        }
                    }
                }

                // 更新状态
                for dep_id in to_update {
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == dep_id) {
                        task.status = TaskStatus::Ready;
                        affected.push(dep_id.clone());
                        tracing::info!("🟢 任务 {} 状态变为 Ready", dep_id);
                    }
                }
            }
            TaskStatus::Failed => {
                // 任务失败，阻塞所有下游任务
                let downstream = graph.get_all_downstream(task_id);

                for dep_id in downstream {
                    if let Some(task) = tasks.iter_mut().find(|t| t.id == dep_id) {
                        if task.status != TaskStatus::Done {
                            task.status = TaskStatus::Blocked;
                            affected.push(dep_id.clone());
                            tracing::warn!("🚫 任务 {} 被阻塞", dep_id);
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(affected)
    }

    /// 获取可以执行的任务
    pub async fn get_ready_tasks(&self, tasks: &[Task]) -> Vec<String> {
        let graph = self.graph.read().await;
        let mut ready = Vec::new();

        for task in tasks {
            if task.status != TaskStatus::Pending && task.status != TaskStatus::Ready {
                continue;
            }

            let deps = graph.get_dependencies(&task.id);
            let all_done = deps.iter().all(|d| {
                tasks
                    .iter()
                    .any(|t| &t.id == d && t.status == TaskStatus::Done)
            });

            if all_done || deps.is_empty() {
                ready.push(task.id.clone());
            }
        }

        ready
    }

    /// 生成依赖报告
    pub async fn generate_report(&self, tasks: &[Task]) -> String {
        let graph = self.graph.read().await;

        let mut report = String::from("📊 依赖关系报告\n\n");

        for task in tasks {
            let deps = graph.get_dependencies(&task.id);
            let dependents = graph.get_dependents(&task.id);

            report.push_str(&format!("{} {}\n", task.id, task.title));
            if !deps.is_empty() {
                report.push_str(&format!("  依赖: {}\n", deps.join(", ")));
            }
            if !dependents.is_empty() {
                report.push_str(&format!("  被依赖: {}\n", dependents.join(", ")));
            }
        }

        // 检查循环
        if graph.has_cycle() {
            report.push_str("\n⚠️ 警告: 存在循环依赖！\n");
        }

        report
    }
}
