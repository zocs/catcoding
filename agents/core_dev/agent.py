"""
🐱 示例 Agent — 英短蓝猫（Core Dev）

演示如何实现一个 CatCoding Agent
"""

import asyncio
import sys
import os

# 添加 base 模块路径
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))

from agent import BaseAgent, AgentMessage


class CoreDevAgent(BaseAgent):
    """英短蓝猫 — 核心开发 Agent"""

    def __init__(self, agent_id: str, project_id: str, workdir: str = "."):
        super().__init__(agent_id, "core_dev", project_id, workdir)

    async def execute_task(self, msg: AgentMessage):
        """执行编码任务"""
        self._log(f"🍳 开始烹饪: {msg.summary}")

        # 模拟任务执行过程
        for i in range(1, 6):
            await asyncio.sleep(1)  # 模拟工作
            self._send_progress(msg.task_id, i * 20, f"进度 {i * 20}%")

        self._log("✅ 烹饪完成！美味！")


async def main():
    """启动 Agent"""
    agent_id = os.environ.get("AGENT_ID", "core-dev-01")
    project_id = os.environ.get("PROJECT_ID", "default")
    workdir = os.environ.get("WORKDIR", ".")

    agent = CoreDevAgent(agent_id, project_id, workdir)
    print(f"🐱 英短蓝猫 Agent 启动: {agent_id}", flush=True)

    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
