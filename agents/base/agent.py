"""
CatCoding Python Agent SDK

猫咪团队的 Python 通信基础库
"""

import asyncio
import json
import sys
from dataclasses import dataclass, field
from enum import Enum
from typing import Optional, List, Dict


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
class TaskDoD:
    """Definition of Done — 任务完成的明确契约

    每个任务必须在分配时指定 DoD，Agent 执行完毕后自动验证。
    原则：Never "figure out what to do" — always "do X, output Y, verify Z"
    """

    output_path: str = ""  # 输出文件路径（必须写入指定位置）
    output_format: str = ""  # 输出格式描述（如 "JSON with fields: ..."）
    verify_command: str = ""  # 验证命令（exit code 0 = success）
    silent_on_success: bool = False  # 成功时静默（cron 任务返回 HEARTBEAT_OK）
    required_artifacts: List[str] = field(default_factory=list)  # 必须产出的文件列表

    def validate(self, workdir: str) -> tuple[bool, str]:
        """验证任务是否真正完成"""
        import os

        # 1. 检查输出文件是否存在
        if self.output_path:
            full_path = (
                os.path.join(workdir, self.output_path)
                if not os.path.isabs(self.output_path)
                else self.output_path
            )
            if not os.path.exists(full_path):
                return False, f"输出文件不存在: {self.output_path}"

        # 2. 检查必需产出物
        for artifact in self.required_artifacts:
            full = (
                os.path.join(workdir, artifact)
                if not os.path.isabs(artifact)
                else artifact
            )
            if not os.path.exists(full):
                return False, f"缺少产出物: {artifact}"

        # 3. 执行验证命令
        if self.verify_command:
            import subprocess

            try:
                result = subprocess.run(
                    self.verify_command,
                    shell=True,
                    cwd=workdir,
                    capture_output=True,
                    text=True,
                    timeout=30,
                )
                if result.returncode != 0:
                    return (
                        False,
                        f"验证失败 (exit {result.returncode}): {result.stderr[:200]}",
                    )
            except subprocess.TimeoutExpired:
                return False, "验证命令超时 (>30s)"

        return True, "ok"

    def to_dict(self) -> dict:
        return {
            "output_path": self.output_path,
            "output_format": self.output_format,
            "verify_command": self.verify_command,
            "silent_on_success": self.silent_on_success,
            "required_artifacts": self.required_artifacts,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "TaskDoD":
        return cls(
            output_path=data.get("output_path", ""),
            output_format=data.get("output_format", ""),
            verify_command=data.get("verify_command", ""),
            silent_on_success=data.get("silent_on_success", False),
            required_artifacts=data.get("required_artifacts", []),
        )


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

        # 启动心跳任务
        heartbeat_task = asyncio.create_task(self._heartbeat_loop())

        try:
            # 读取 stdin（非阻塞）
            reader = asyncio.StreamReader()
            protocol = asyncio.StreamReaderProtocol(reader)
            await asyncio.get_event_loop().connect_read_pipe(
                lambda: protocol, sys.stdin.buffer
            )

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

        # 解析 DoD（从 details 字段中读取 JSON）
        dod = None
        if msg.details:
            try:
                details = json.loads(msg.details)
                if "dod" in details:
                    dod = TaskDoD.from_dict(details["dod"])
                    self._log(
                        f"  📝 DoD: output={dod.output_path}, verify={dod.verify_command or 'none'}"
                    )
            except (json.JSONDecodeError, KeyError):
                pass

        # 发送进度
        self._send_progress(msg.task_id, 0, "开始处理任务")

        # 子类实现具体逻辑
        try:
            await self.execute_task(msg)

            # DoD 验证
            if dod:
                ok, reason = dod.validate(self.workdir)
                if not ok:
                    self._send_result(msg.task_id, "failed", f"DoD 验证失败: {reason}")
                    self._log(f"  ❌ DoD 验证失败: {reason}")
                else:
                    if dod.silent_on_success:
                        self._send_result(msg.task_id, "completed", "HEARTBEAT_OK")
                    else:
                        self._send_result(msg.task_id, "completed", "任务完成 (DoD ✓)")
                    self._log("  ✅ DoD 验证通过")
            else:
                self._send_result(msg.task_id, "completed", "任务完成")
                self._log("  ⚠️ 无 DoD — 无法客观验证完成状态")
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
