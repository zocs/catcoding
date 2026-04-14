"""
🐱 暹罗猫 — PM Agent

职责：
- 理解用户需求、拆分任务
- 定义任务依赖关系
- 监控整体进度
- 处理阻塞和升级
- 向用户汇报

PM 决策原则（铁律）：
1. 参照顶级软件工程思维（YAGNI、DRY、SOLID）
2. 不允许"跑不通就硬编码应付"的三流思维
3. 不全凭经验——IT瞬息万变，经验可能过时
4. 每个技术决策必须有依据
"""

import asyncio
import json
import os
import sys
from typing import List, Dict, Any, Optional
from dataclasses import dataclass, field, asdict

sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'base'))
from agent import BaseAgent, AgentMessage, MessageType, TaskStatus


@dataclass
class TaskPlan:
    """任务计划"""
    id: str
    title: str
    description: str
    role: str  # 分配给哪个角色
    depends_on: List[str] = field(default_factory=list)
    priority: int = 0  # 0=普通, 1=高, 2=紧急


@dataclass
class ProjectSummary:
    """项目摘要"""
    name: str
    total_tasks: int
    completed: int
    in_progress: int
    blocked: int
    failed: int
    progress_percent: float


class PMAgent(BaseAgent):
    """暹罗猫 PM Agent — 全局观、调度、汇报"""

    def __init__(self, agent_id: str, project_id: str, workdir: str = "."):
        super().__init__(agent_id, "pm", project_id, workdir)
        self.tasks: Dict[str, TaskPlan] = {}
        self.task_status: Dict[str, str] = {}

    async def execute_task(self, msg: AgentMessage):
        """执行 PM 任务（需求分析、任务拆分）"""
        self._log(f"📋 分析需求: {msg.summary}")

        # 分析需求，生成任务计划
        tasks = await self._analyze_requirements(msg.summary)

        # 输出任务计划
        self._log(f"📋 拆分出 {len(tasks)} 个子任务")
        for task in tasks:
            self._log(f"  - [{task.role}] {task.title}")
            self.tasks[task.id] = task

        # 返回任务计划
        plan_json = json.dumps([asdict(t) for t in tasks], ensure_ascii=False)
        self._send_result(msg.task_id, "completed", f"任务拆分完成: {len(tasks)} 个子任务")

    async def _analyze_requirements(self, requirement: str) -> List[TaskPlan]:
        """分析需求，拆分任务"""
        tasks = []

        # 简化版：基于关键词分析
        req_lower = requirement.lower()

        # 检测任务类型
        if "登录" in req_lower or "login" in req_lower or "auth" in req_lower:
            tasks = self._plan_auth_feature(requirement)
        elif "api" in req_lower or "接口" in req_lower:
            tasks = self._plan_api_feature(requirement)
        elif "前端" in req_lower or "ui" in req_lower or "页面" in req_lower:
            tasks = self._plan_frontend_feature(requirement)
        elif "测试" in req_lower or "test" in req_lower:
            tasks = self._plan_test_task(requirement)
        else:
            # 通用任务拆分
            tasks = self._plan_generic_task(requirement)

        return tasks

    def _plan_auth_feature(self, requirement: str) -> List[TaskPlan]:
        """规划认证功能任务"""
        return [
            TaskPlan(
                id="auth-001",
                title="设计认证接口",
                description="设计登录/注册/Token 刷新的 API 接口",
                role="backend",
                priority=1,
            ),
            TaskPlan(
                id="auth-002",
                title="实现认证后端",
                description="实现 JWT 认证逻辑、用户模型、密码加密",
                role="backend",
                depends_on=["auth-001"],
                priority=1,
            ),
            TaskPlan(
                id="auth-003",
                title="实现登录页面",
                description="Vue3 登录表单、表单验证、错误处理",
                role="frontend",
                depends_on=["auth-001"],
            ),
            TaskPlan(
                id="auth-004",
                title="实现认证流程",
                description="前端 Token 管理、路由守卫、自动刷新",
                role="frontend",
                depends_on=["auth-002", "auth-003"],
            ),
            TaskPlan(
                id="auth-005",
                title="编写认证测试",
                description="单元测试 + 集成测试",
                role="tester",
                depends_on=["auth-002", "auth-004"],
            ),
            TaskPlan(
                id="auth-006",
                title="代码审查",
                description="审查认证模块的安全性、代码质量",
                role="reviewer",
                depends_on=["auth-004", "auth-005"],
            ),
        ]

    def _plan_api_feature(self, requirement: str) -> List[TaskPlan]:
        """规划 API 功能任务"""
        return [
            TaskPlan(
                id="api-001",
                title="设计 API 接口",
                description="定义端点、请求/响应格式",
                role="backend",
                priority=1,
            ),
            TaskPlan(
                id="api-002",
                title="实现 API 后端",
                description="实现业务逻辑、数据验证",
                role="backend",
                depends_on=["api-001"],
            ),
            TaskPlan(
                id="api-003",
                title="编写 API 测试",
                description="接口测试、边界测试",
                role="tester",
                depends_on=["api-002"],
            ),
        ]

    def _plan_frontend_feature(self, requirement: str) -> List[TaskPlan]:
        """规划前端功能任务"""
        return [
            TaskPlan(
                id="fe-001",
                title="设计页面结构",
                description="组件划分、数据流设计",
                role="frontend",
                priority=1,
            ),
            TaskPlan(
                id="fe-002",
                title="实现页面组件",
                description="Vue3 组件实现",
                role="frontend",
                depends_on=["fe-001"],
            ),
            TaskPlan(
                id="fe-003",
                title="UI 审查",
                description="检查 UI 一致性、响应式",
                role="reviewer",
                depends_on=["fe-002"],
            ),
        ]

    def _plan_test_task(self, requirement: str) -> List[TaskPlan]:
        """规划测试任务"""
        return [
            TaskPlan(
                id="test-001",
                title="编写测试用例",
                description="分析代码，编写测试",
                role="tester",
            ),
            TaskPlan(
                id="test-002",
                title="执行测试",
                description="运行测试，收集覆盖率",
                role="tester",
                depends_on=["test-001"],
            ),
        ]

    def _plan_generic_task(self, requirement: str) -> List[TaskPlan]:
        """通用任务拆分"""
        return [
            TaskPlan(
                id="task-001",
                title="需求分析",
                description=requirement,
                role="core_dev",
                priority=1,
            ),
            TaskPlan(
                id="task-002",
                title="实现",
                description="实现需求",
                role="core_dev",
                depends_on=["task-001"],
            ),
            TaskPlan(
                id="task-003",
                title="审查",
                description="代码审查",
                role="reviewer",
                depends_on=["task-002"],
            ),
        ]

    def generate_progress_report(self) -> str:
        """生成进度报告"""
        total = len(self.tasks)
        completed = sum(1 for s in self.task_status.values() if s == "done")
        in_progress = sum(1 for s in self.task_status.values() if s == "active")
        blocked = sum(1 for s in self.task_status.values() if s == "blocked")
        failed = sum(1 for s in self.task_status.values() if s == "failed")

        percent = (completed / total * 100) if total > 0 else 0

        report = f"""
🐱 猫咪团队进度报告

📊 总体进度: {percent:.1f}% ({completed}/{total})
  ✅ 已完成: {completed}
  🔵 进行中: {in_progress}
  🚫 被阻塞: {blocked}
  ❌ 失败: {failed}

📋 任务详情:
"""
        for task_id, task in self.tasks.items():
            status = self.task_status.get(task_id, "pending")
            emoji = {
                "pending": "⏳",
                "active": "🔵",
                "done": "✅",
                "blocked": "🚫",
                "failed": "❌",
            }.get(status, "❓")
            report += f"  {emoji} [{task.role}] {task.title}\n"

        return report


async def main():
    """启动 PM Agent"""
    agent_id = os.environ.get("AGENT_ID", "pm-agent-01")
    project_id = os.environ.get("PROJECT_ID", "default")
    workdir = os.environ.get("WORKDIR", ".")

    agent = PMAgent(agent_id, project_id, workdir)
    print(f"🐱 暹罗猫 PM Agent 启动: {agent_id}", flush=True)

    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
