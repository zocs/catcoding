#!/usr/bin/env python3
"""
🐱 CatCoding Agent Runner

统一入口：python3 run_agent.py <role> [--agent-id ID] [--project-id ID] [--workdir DIR]

支持的角色: pm, core_dev, frontend, backend, reviewer, tester, deploy, tech_scout
"""

import argparse
import asyncio
import os
import sys

# 添加 agents 目录到路径
AGENTS_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, AGENTS_DIR)
sys.path.insert(0, os.path.join(AGENTS_DIR, "base"))

ROLE_MODULES = {
    "pm": ("pm.agent", "PMAgent"),
    "core_dev": ("core_dev.agent", "CoreDevAgent"),
    "frontend": ("frontend.agent", "FrontendAgent"),
    "backend": ("backend.agent", "BackendAgent"),
    "reviewer": ("reviewer.agent", "ReviewerAgent"),
    "tester": ("tester.agent", "TesterAgent"),
    "deploy": ("deploy.agent", "DeployAgent"),
    "tech_scout": ("tech_scout.agent", "TechScoutAgent"),
}


def load_agent_class(role: str):
    """动态加载 Agent 类"""
    if role not in ROLE_MODULES:
        print(f"❌ 未知角色: {role}", file=sys.stderr)
        print(f"   可用角色: {', '.join(ROLE_MODULES.keys())}", file=sys.stderr)
        sys.exit(1)

    module_path, class_name = ROLE_MODULES[role]

    # 尝试导入
    try:
        module = __import__(module_path.replace("/", ".").replace(".agent", ".agent"), fromlist=[class_name])
        return getattr(module, class_name)
    except (ImportError, ModuleNotFoundError):
        # fallback: 用 BaseAgent
        print(f"⚠️  未找到 {role} 的具体实现，使用 BaseAgent", file=sys.stderr)
        from agent import BaseAgent

        class GenericAgent(BaseAgent):
            async def execute_task(self, msg):
                self._log(f"🔧 执行通用任务: {msg.summary}")
                for i in range(1, 6):
                    await asyncio.sleep(1)
                    self._send_progress(msg.task_id, i * 20, f"进度 {i * 20}%")
                self._log("✅ 任务完成")

        GenericAgent.__name__ = f"{role.title()}Agent"
        return GenericAgent


def main():
    parser = argparse.ArgumentParser(description="🐱 CatCoding Agent Runner")
    parser.add_argument("role", help="Agent 角色 (pm, core_dev, frontend, ...)")
    parser.add_argument("--agent-id", default=None, help="Agent ID")
    parser.add_argument("--project-id", default="default", help="项目 ID")
    parser.add_argument("--workdir", default=".", help="工作目录")

    args = parser.parse_args()

    agent_id = args.agent_id or os.environ.get("AGENT_ID", f"{args.role}-01")
    project_id = args.project_id or os.environ.get("PROJECT_ID", "default")
    workdir = args.workdir or os.environ.get("WORKDIR", ".")

    AgentClass = load_agent_class(args.role)
    agent = AgentClass(agent_id, project_id, workdir)

    print(f"🐱 {args.role} Agent 启动: {agent_id}", flush=True)
    asyncio.run(agent.run())


if __name__ == "__main__":
    main()
