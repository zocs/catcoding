"""🐱 英短蓝猫 — Core Dev Agent

Honest scaffold: writes a real stub file; does not pretend to generate code.
Swap `scaffold_task` for an LLM-backed implementation to enable real output.
"""

import asyncio
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent, AgentMessage
from scaffold import scaffold_task


class CoreDevAgent(BaseAgent):
    def __init__(self, agent_id: str, project_id: str, workdir: str = "."):
        super().__init__(agent_id, "core_dev", project_id, workdir)

    async def execute_task(self, msg: AgentMessage):
        self._log(f"🍳 开始烹饪 (scaffold mode): {msg.summary}")
        await scaffold_task(
            self,
            msg,
            [
                "分析需求",
                "设计模块边界",
                "生成骨架代码",
                "写测试桩",
                "落盘产物",
            ],
        )
        self._log("✅ 骨架完成（非 LLM 真实代码生成）")


async def main():
    agent = CoreDevAgent(
        os.environ.get("AGENT_ID", "core-dev-01"),
        os.environ.get("PROJECT_ID", "default"),
        os.environ.get("WORKDIR", "."),
    )
    print(f"🐱 英短蓝猫 Agent 启动 (scaffold mode): {agent.agent_id}", flush=True)
    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
