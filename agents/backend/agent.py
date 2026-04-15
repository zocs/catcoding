"""🐱 缅因猫 — 后端开发 Agent"""
import asyncio
import sys, os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'base'))
from agent import BaseAgent, AgentMessage

class BackendAgent(BaseAgent):
    def __init__(self, agent_id, project_id, workdir="."):
        super().__init__(agent_id, "backend", project_id, workdir)

    async def execute_task(self, msg):
        self._log(f"🔧 后端任务: {msg.summary}")
        steps = ["分析 API 需求", "设计数据模型", "实现业务逻辑", "编写测试", "性能优化"]
        for i, step in enumerate(steps, 1):
            await asyncio.sleep(1)
            self._send_progress(msg.task_id, i * 20, step)
        self._log("✅ 后端任务完成")

async def main():
    agent = BackendAgent(
        os.environ.get("AGENT_ID", "backend-01"),
        os.environ.get("PROJECT_ID", "default"),
        os.environ.get("WORKDIR", "."),
    )
    print(f"🐱 缅因猫 Agent 启动: {agent.agent_id}", flush=True)
    await agent.run()

if __name__ == "__main__":
    asyncio.run(main())
