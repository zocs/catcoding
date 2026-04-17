"""
CatCoding Agent NATS 通信库

通过 NATS 与 Daemon 通信（备选方案，stdin/stdout 是主要方式）
"""

import json
from typing import Optional, Callable, Awaitable

try:
    import nats
    from nats.aio.client import Client as NATS

    HAS_NATS = True
except ImportError:
    HAS_NATS = False


class AgentComm:
    """Agent 通信客户端"""

    def __init__(self, nats_url: str = "nats://127.0.0.1:4222"):
        self.nats_url = nats_url
        self.client: Optional[NATS] = None
        self._handlers: dict[str, Callable] = {}

    async def connect(self):
        """连接到 NATS"""
        if not HAS_NATS:
            raise ImportError("请安装 nats-py: pip install nats-py")

        self.client = await nats.connect(self.nats_url)
        print(f"📡 已连接 NATS: {self.nats_url}")

    async def disconnect(self):
        """断开连接"""
        if self.client:
            await self.client.drain()
            await self.client.close()

    async def publish(self, subject: str, data: dict):
        """发布消息"""
        if not self.client:
            raise RuntimeError("未连接到 NATS")
        payload = json.dumps(data).encode()
        await self.client.publish(subject, payload)

    async def subscribe(self, subject: str, handler: Callable[[dict], Awaitable[None]]):
        """订阅主题"""
        if not self.client:
            raise RuntimeError("未连接到 NATS")

        async def _handler(msg):
            try:
                data = json.loads(msg.data.decode())
                await handler(data)
            except Exception as e:
                print(f"处理消息失败: {e}")

        sub = await self.client.subscribe(subject, cb=_handler)
        self._handlers[subject] = sub
        print(f"📡 已订阅: {subject}")
        return sub

    async def request(self, subject: str, data: dict, timeout: float = 5.0) -> dict:
        """请求-响应模式"""
        if not self.client:
            raise RuntimeError("未连接到 NATS")
        payload = json.dumps(data).encode()
        response = await self.client.request(subject, payload, timeout=timeout)
        return json.loads(response.data.decode())


# NATS Subject 定义
class Subjects:
    """NATS Subject 路由"""

    @staticmethod
    def tasks(role: str) -> str:
        return f"tasks.{role}"

    @staticmethod
    def progress(agent_id: str) -> str:
        return f"agent.{agent_id}.progress"

    @staticmethod
    def heartbeat() -> str:
        return "agent.heartbeat"

    @staticmethod
    def alert() -> str:
        return "watchdog.alert"

    @staticmethod
    def logs(project_id: str) -> str:
        return f"logs.{project_id}"
