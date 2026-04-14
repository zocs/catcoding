"""
CatCoding Python Agent SDK

猫咪团队的 Python 通信基础库
"""

import asyncio
import json
import sys
import time
from dataclasses import dataclass, field, asdict
from enum import Enum
from typing import Optional, List, Dict, Any


class TaskStatus(str, Enum):
    """任务状态（8 态状态机）"""
    PENDING = "pending"
    BLOCKED = "blocked"
    READY = "ready"
    ACTIVE = "active"
    REVIEWING = "reviewing"
    DONE = "done"
    ROLLBACKED = "rollbacked"
    FAILED = "failed"


class MessageType(str, Enum):
    """消息类型"""
    TASK_RESULT = "task_result"
    STATUS_UPDATE = "status_update"
    REQUEST = "request"
    ALERT = "alert"
    HEARTBEAT = "heartbeat"


@dataclass
class AgentMessage:
    """Agent 间通信消息"""
    msg_id: str = ""
    type: str = ""
    from_agent: str = ""
    to: str = ""
    timestamp: str = ""
    task_id: Optional[str] = None
    status: Optional[str] = None
    progress_percent: Optional[int] = None
    artifacts: List[str] = field(default_factory=list)
    summary: str = ""
    details: Optional[str] = None
    blockers: List[str] = field(default_factory=list)

    def to_json(self) -> str:
        import uuid
        from datetime import datetime, timezone
        if not self.msg_id:
            self.msg_id = str(uuid.uuid4())
        if not self.timestamp:
            self.timestamp = datetime.now(timezone.utc).isoformat()
        data = {
            "msg_id": self.msg_id,
            "type": self.type,
            "from": self.from_agent,
            "to": self.to,
            "timestamp": self.timestamp,
            "task_id": self.task_id,
            "status": self.status,
            "progress_percent": self.progress_percent,
            "artifacts": self.artifacts,
            "summary": self.summary,
            "details": self.details,
            "blockers": self.blockers,
        }
        return json.dumps(data)

    @classmethod
    def from_json(cls, data: str) -> "AgentMessage":
        d = json.loads(data)
        return cls(
            msg_id=d.get("msg_id", ""),
            type=d.get("type", ""),
            from_agent=d.get("from", ""),
            to=d.get("to", ""),
            timestamp=d.get("timestamp", ""),
            task_id=d.get("task_id"),
            status=d.get("status"),
            progress_percent=d.get("progress_percent"),
            artifacts=d.get("artifacts", []),
            summary=d.get("summary", ""),
            details=d.get("details"),
            blockers=d.get("blockers", []),
        )


class BaseAgent:
    """Agent 基类 — 所有猫咪的父类"""

    def __init__(self, agent_id: str, role: str, project_id: str, workdir: str = "."):
        self.agent_id = agent_id
        self.role = role
        self.project_id = project_id
        self.workdir = workdir
        self.running = True
        self.current_task: Optional[Dict] = None

    async def run(self):
        """主循环：读取 stdin，处理消息，发送心跳"""
        import uuid
        from datetime import datetime, timezone

        # 启动心跳任务
        heartbeat_task = asyncio.create_task(self._heartbeat_loop())

        try:
            # 读取 stdin（非阻塞）
            reader = asyncio.StreamReader()
            protocol = asyncio.StreamReaderProtocol(reader)
            await asyncio.get_event_loop().connect_read_pipe(lambda: protocol, sys.stdin.buffer)

            while self.running:
                try:
                    line = await asyncio.wait_for(reader.readline(), timeout=1.0)
                    if not line:
                        break
                    message = line.decode().strip()
                    if message:
                        await self._handle_message(message)
                except asyncio.TimeoutError:
                    continue
                except Exception as e:
                    self._log(f"读取消息错误: {e}")
        finally:
            heartbeat_task.cancel()

    async def _heartbeat_loop(self):
        """心跳循环 — 每 5 秒发送一次"""
        while self.running:
            heartbeat = AgentMessage(
                type=MessageType.HEARTBEAT,
                from_agent=self.agent_id,
                to="watchdog",
                summary="alive",
            )
            self._send(heartbeat.to_json())
            await asyncio.sleep(5)

    async def _handle_message(self, raw: str):
        """处理收到的消息"""
        try:
            msg = AgentMessage.from_json(raw)
            self._log(f"收到消息: {msg.type} - {msg.summary}")

            if msg.type == "task.assign":
                await self._handle_task_assign(msg)
            elif msg.type == "stop":
                self.running = False
            else:
                self._log(f"未知消息类型: {msg.type}")
        except Exception as e:
            self._log(f"处理消息失败: {e}")

    async def _handle_task_assign(self, msg: AgentMessage):
        """处理任务分配"""
        self._log(f"📋 收到任务: {msg.summary}")
        self.current_task = {"id": msg.task_id, "summary": msg.summary}

        # 发送进度
        self._send_progress(msg.task_id, 0, "开始处理任务")

        # 子类实现具体逻辑
        try:
            await self.execute_task(msg)
            self._send_result(msg.task_id, "completed", "任务完成")
        except Exception as e:
            self._send_result(msg.task_id, "failed", f"任务失败: {e}")

        self.current_task = None

    async def execute_task(self, msg: AgentMessage):
        """执行任务 — 子类需重写此方法"""
        raise NotImplementedError("子类必须实现 execute_task 方法")

    def _send(self, data: str):
        """发送消息到 stdout"""
        print(data, flush=True)

    def _send_progress(self, task_id: str, percent: int, detail: str):
        """发送进度更新"""
        msg = AgentMessage(
            type=MessageType.STATUS_UPDATE,
            from_agent=self.agent_id,
            to="pm-agent",
            task_id=task_id,
            status="progress",
            progress_percent=percent,
            summary=detail,
        )
        self._send(msg.to_json())

    def _send_result(self, task_id: str, status: str, summary: str):
        """发送任务结果"""
        msg = AgentMessage(
            type=MessageType.TASK_RESULT,
            from_agent=self.agent_id,
            to="pm-agent",
            task_id=task_id,
            status=status,
            progress_percent=100 if status == "completed" else None,
            summary=summary,
        )
        self._send(msg.to_json())

    def _log(self, message: str):
        """日志输出"""
        from datetime import datetime, timezone
        ts = datetime.now(timezone.utc).strftime("%H:%M:%S")
        print(f"[{ts}] 🐱 [{self.agent_id}] {message}", file=sys.stderr, flush=True)
