"""🐱 橘猫 — Frontend Agent (honest scaffold)"""

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent
from scaffold import scaffold_task


class FrontendAgent(BaseAgent):
    def __init__(self, agent_id, project_id, workdir="."):
        super().__init__(agent_id, "frontend", project_id, workdir)

    async def execute_task(self, msg):
        self._log(f"🎨 前端任务 (scaffold mode): {msg.summary}")
        await scaffold_task(
            self,
            msg,
            ["分析需求", "设计组件结构", "生成模板骨架", "样式占位", "落盘产物"],
        )
        self._log("✅ 前端骨架完成")


async def main():
    agent = FrontendAgent(
        os.environ.get("AGENT_ID", "frontend-01"),
        os.environ.get("PROJECT_ID", "default"),
        os.environ.get("WORKDIR", "."),
    )
    print(f"🐱 橘猫 Agent 启动 (scaffold mode): {agent.agent_id}", flush=True)
    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
