"""🐱 橘猫 — 前端开发 Agent"""

import asyncio
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent


class FrontendAgent(BaseAgent):
    def __init__(self, agent_id, project_id, workdir="."):
        super().__init__(agent_id, "frontend", project_id, workdir)

    async def execute_task(self, msg):
        self._log(f"🎨 前端任务: {msg.summary}")
        steps = ["分析需求", "设计组件结构", "编写模板", "添加样式", "测试交互"]
        for i, step in enumerate(steps, 1):
            await asyncio.sleep(1)
            self._send_progress(msg.task_id, i * 20, step)
        self._log("✅ 前端任务完成")


async def main():
    agent = FrontendAgent(
        os.environ.get("AGENT_ID", "frontend-01"),
        os.environ.get("PROJECT_ID", "default"),
        os.environ.get("WORKDIR", "."),
    )
    print(f"🐱 橘猫 Agent 启动: {agent.agent_id}", flush=True)
    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
