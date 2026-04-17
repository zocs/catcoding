"""🐱 狸花猫 — Deploy Agent (honest scaffold)"""

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent
from scaffold import scaffold_task


class DeployAgent(BaseAgent):
    def __init__(self, agent_id, project_id, workdir="."):
        super().__init__(agent_id, "deploy", project_id, workdir)

    async def execute_task(self, msg):
        self._log(f"🚀 部署任务 (scaffold mode): {msg.summary}")
        await scaffold_task(
            self,
            msg,
            ["检查构建产物", "冒烟验证", "准备部署清单", "落盘部署描述"],
        )
        self._log("✅ 部署骨架完成")


async def main():
    agent = DeployAgent(
        os.environ.get("AGENT_ID", "deploy-01"),
        os.environ.get("PROJECT_ID", "default"),
        os.environ.get("WORKDIR", "."),
    )
    print(f"🐱 狸花猫 Agent 启动 (scaffold mode): {agent.agent_id}", flush=True)
    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
