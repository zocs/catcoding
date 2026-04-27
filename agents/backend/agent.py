"""🐱 缅因猫 — Backend Agent (honest scaffold)"""

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent
from scaffold import scaffold_task


class BackendAgent(BaseAgent):
    def __init__(self, agent_id, project_id, workdir="."):
        super().__init__(agent_id, "backend", project_id, workdir)

    async def execute_task(self, msg):
        self._log(f"🐟 后端任务 (scaffold mode): {msg.summary}")
        await scaffold_task(
            self,
            msg,
            [
                "分析接口契约",
                "设计数据模型",
                "生成路由骨架",
                "写占位 handler",
                "落盘产物",
            ],
        )
        self._log("✅ 后端骨架完成")


async def main():
    agent = BackendAgent(
        os.environ.get("AGENT_ID", "backend-01"),
        os.environ.get("PROJECT_ID", "default"),
        os.environ.get("WORKDIR", "."),
    )
    print(f"🐱 缅因猫 Agent 启动 (scaffold mode): {agent.agent_id}", flush=True)
    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
