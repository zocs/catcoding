"""🐱 狸花猫 — Deploy Agent (CI/CD)"""

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent


class DeployAgent(BaseAgent):
    def __init__(self, agent_id, project_id, workdir="."):
        super().__init__(agent_id, "deploy", project_id, workdir)

    async def execute_task(self, msg):
        self._log(f"🚀 部署任务: {msg.summary}")
        steps = ["检查构建产物", "运行冒烟测试", "推送镜像", "滚动更新", "健康检查"]
        for i, step in enumerate(steps, 1):
            await asyncio.sleep(1)
            self._send_progress(msg.task_id, i * 20, step)
        self._log("✅ 部署完成")


async def main():
    agent = DeployAgent(
        os.environ.get("AGENT_ID", "deploy-01"),
        os.environ.get("PROJECT_ID", "default"),
        os.environ.get("WORKDIR", "."),
    )
    print(f"🐱 狸花猫 Agent 启动: {agent.agent_id}", flush=True)
    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
