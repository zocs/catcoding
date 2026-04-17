"""
CatCoding Python Agent SDK 测试
"""

import pytest
import sys
import os

# 添加 agents 目录到路径
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', '..', 'agents'))

from base.agent import TaskStatus, MessageType, TaskDoD


def test_task_status_enum():
    """测试任务状态枚举"""
    assert TaskStatus.PENDING.value == "pending"
    assert TaskStatus.BLOCKED.value == "blocked"
    assert TaskStatus.READY.value == "ready"
    assert TaskStatus.ACTIVE.value == "active"
    assert TaskStatus.REVIEWING.value == "reviewing"
    assert TaskStatus.DONE.value == "done"
    assert TaskStatus.ROLLBACKED.value == "rollbacked"
    assert TaskStatus.FAILED.value == "failed"


def test_message_type_enum():
    """测试消息类型枚举"""
    assert MessageType.TASK_RESULT.value == "task_result"
    assert MessageType.STATUS_UPDATE.value == "status_update"
    assert MessageType.REQUEST.value == "request"
    assert MessageType.ALERT.value == "alert"
    assert MessageType.HEARTBEAT.value == "heartbeat"


def test_task_dod_creation():
    """测试任务 DoD 创建"""
    dod = TaskDoD(
        output_path="/tmp/test.txt",
        output_format="Plain text",
        verify_command="test -f /tmp/test.txt",
        silent_on_success=True,
        required_artifacts=["/tmp/test.txt"]
    )
    
    assert dod.output_path == "/tmp/test.txt"
    assert dod.output_format == "Plain text"
    assert dod.verify_command == "test -f /tmp/test.txt"
    assert dod.silent_on_success is True
    assert dod.required_artifacts == ["/tmp/test.txt"]


def test_task_dod_defaults():
    """测试任务 DoD 默认值"""
    dod = TaskDoD()
    
    assert dod.output_path == ""
    assert dod.output_format == ""
    assert dod.verify_command == ""
    assert dod.silent_on_success is False
    assert dod.required_artifacts == []


def test_task_status_from_string():
    """测试从字符串创建任务状态"""
    # 这个测试需要检查 BaseAgent 的 from_str 方法
    # 暂时跳过，因为需要导入 BaseAgent
    pass


if __name__ == "__main__":
    pytest.main([__file__, "-v"])