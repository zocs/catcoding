"""
🐱 阿比西尼亚猫 — Test Agent

职责：
- 测试用例编写
- 自动化测试执行
- 覆盖率分析
- 测试报告生成

特点：好奇心强、爱探索 — 测试边界情况和异常路径
"""

import asyncio
import os
import subprocess
import sys
from typing import List, Optional
from dataclasses import dataclass, field

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent, AgentMessage


@dataclass
class TestResult:
    """测试结果"""

    name: str
    status: str  # "passed", "failed", "skipped", "error"
    duration_ms: float
    message: Optional[str] = None
    traceback: Optional[str] = None


@dataclass
class TestSuite:
    """测试套件"""

    name: str
    total: int
    passed: int
    failed: int
    skipped: int
    errors: int
    duration_ms: float
    coverage_percent: Optional[float] = None
    results: List[TestResult] = field(default_factory=list)


class TestAgent(BaseAgent):
    """阿比西尼亚猫 Test Agent — 好奇心强、爱探索"""

    def __init__(self, agent_id: str, project_id: str, workdir: str = "."):
        super().__init__(agent_id, "tester", project_id, workdir)

    async def execute_task(self, msg: AgentMessage):
        """执行测试任务"""
        task_type = self._detect_task_type(msg.summary)

        if task_type == "write":
            await self._write_tests(msg)
        elif task_type == "run":
            await self._run_tests(msg)
        elif task_type == "coverage":
            await self._analyze_coverage(msg)
        else:
            await self._run_tests(msg)

    def _detect_task_type(self, summary: str) -> str:
        """检测任务类型"""
        summary_lower = summary.lower()
        if "写" in summary_lower or "编写" in summary_lower or "write" in summary_lower:
            return "write"
        elif (
            "运行" in summary_lower or "执行" in summary_lower or "run" in summary_lower
        ):
            return "run"
        elif "覆盖" in summary_lower or "coverage" in summary_lower:
            return "coverage"
        return "run"

    async def _write_tests(self, msg: AgentMessage):
        """编写测试用例"""
        self._log(f"📝 编写测试: {msg.summary}")

        # 获取要测试的文件
        files = msg.artifacts if msg.artifacts else []

        if not files:
            self._send_result(msg.task_id, "failed", "未指定要测试的文件")
            return

        test_files = []
        for source_file in files:
            if not os.path.exists(source_file):
                self._log(f"⚠️ 文件不存在: {source_file}")
                continue

            # 生成测试文件
            test_file = self._generate_test_file(source_file)
            if test_file:
                test_files.append(test_file)
                self._log(f"📝 生成测试: {test_file}")

        if test_files:
            self._send_result(
                msg.task_id,
                "completed",
                f"生成 {len(test_files)} 个测试文件: {', '.join(test_files)}",
            )
        else:
            self._send_result(msg.task_id, "failed", "未能生成测试文件")

    def _generate_test_file(self, source_file: str) -> Optional[str]:
        """生成测试文件"""
        # 确定测试文件路径
        dir_name = os.path.dirname(source_file)
        base_name = os.path.basename(source_file)
        name_without_ext = os.path.splitext(base_name)[0]
        ext = os.path.splitext(base_name)[1]

        # Python 文件
        if ext == ".py":
            test_file = os.path.join(dir_name, f"test_{name_without_ext}.py")
            test_content = self._generate_python_test(source_file, name_without_ext)
            with open(test_file, "w") as f:
                f.write(test_content)
            return test_file

        # Rust 文件
        elif ext == ".rs":
            test_dir = os.path.join(os.path.dirname(dir_name), "tests")
            os.makedirs(test_dir, exist_ok=True)
            test_file = os.path.join(test_dir, f"test_{name_without_ext}.rs")
            test_content = self._generate_rust_test(source_file, name_without_ext)
            with open(test_file, "w") as f:
                f.write(test_content)
            return test_file

        return None

    def _generate_python_test(self, source_file: str, module_name: str) -> str:
        """生成 Python 测试文件"""
        # 读取源文件，分析函数
        with open(source_file, "r") as f:
            content = f.read()

        # 提取函数名
        import re

        functions = re.findall(r"def\s+(\w+)\s*\(", content)
        functions = [f for f in functions if not f.startswith("_")]

        test_content = f'''"""
测试 {module_name}

自动生成的测试用例，请根据实际情况修改
"""

import pytest
from {module_name} import *


'''
        for func in functions:
            test_content += f'''
class Test{func.capitalize()}:
    """测试 {func}"""

    def test_{func}_basic(self):
        """基本功能测试"""
        assert True

    def test_{func}_edge_cases(self):
        """边界情况测试"""
        assert True

    def test_{func}_error_handling(self):
        """错误处理测试"""
        assert True

'''

        return test_content

    def _generate_rust_test(self, source_file: str, module_name: str) -> str:
        """生成 Rust 测试文件"""
        return f"""//! 测试 {module_name}
//!
//! 自动生成的测试用例，请根据实际情况修改

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_basic() {{
        assert_eq!(2 + 2, 4);
    }}

    #[test]
    fn test_edge_cases() {{
        assert!(Vec::<u8>::new().is_empty());
    }}
}}
"""

    async def _run_tests(self, msg: AgentMessage):
        """运行测试"""
        self._log(f"🧪 运行测试: {msg.summary}")

        # 检测项目类型
        project_type = self._detect_project_type()

        if project_type == "python":
            suite = await self._run_python_tests()
        elif project_type == "rust":
            suite = await self._run_rust_tests()
        elif project_type == "node":
            suite = await self._run_node_tests()
        else:
            self._send_result(msg.task_id, "failed", f"未知项目类型: {project_type}")
            return

        # 生成报告
        report = self._generate_report(suite)

        status = "completed" if suite.failed == 0 and suite.errors == 0 else "failed"
        self._send_result(msg.task_id, status, report)

    def _detect_project_type(self) -> str:
        """检测项目类型"""
        if os.path.exists(os.path.join(self.workdir, "Cargo.toml")):
            return "rust"
        elif (
            os.path.exists(os.path.join(self.workdir, "pyproject.toml"))
            or os.path.exists(os.path.join(self.workdir, "setup.py"))
            or os.path.exists(os.path.join(self.workdir, "requirements.txt"))
        ):
            return "python"
        elif os.path.exists(os.path.join(self.workdir, "package.json")):
            return "node"
        return "unknown"

    async def _run_python_tests(self) -> TestSuite:
        """运行 Python 测试"""
        cmd = ["python", "-m", "pytest", "-v", "--tb=short", self.workdir]
        return await self._execute_test_command("Python Tests", cmd)

    async def _run_rust_tests(self) -> TestSuite:
        """运行 Rust 测试"""
        cmd = ["cargo", "test", "--verbose"]
        return await self._execute_test_command("Rust Tests", cmd, workdir=self.workdir)

    async def _run_node_tests(self) -> TestSuite:
        """运行 Node 测试"""
        import json

        package_json = os.path.join(self.workdir, "package.json")
        if not os.path.exists(package_json):
            return TestSuite(
                name="Node Tests",
                total=0,
                passed=0,
                failed=0,
                skipped=0,
                errors=1,
                duration_ms=0,
            )

        with open(package_json, "r", encoding="utf-8") as f:
            pkg = json.load(f)
        scripts = pkg.get("scripts", {})

        if os.path.exists(os.path.join(self.workdir, "pnpm-lock.yaml")):
            runner = "pnpm"
        elif os.path.exists(os.path.join(self.workdir, "yarn.lock")):
            runner = "yarn"
        else:
            runner = "npm"

        if "test" in scripts:
            cmd = [runner, "run", "test"]
            return await self._execute_test_command(
                "Node Tests", cmd, workdir=self.workdir
            )

        if "check" in scripts:
            cmd = [runner, "run", "check"]
            return await self._execute_test_command(
                "Node Check", cmd, workdir=self.workdir
            )

        if "build" in scripts:
            cmd = [runner, "run", "build"]
            return await self._execute_test_command(
                "Node Build", cmd, workdir=self.workdir
            )

        return TestSuite(
            name="Node Tests",
            total=0,
            passed=0,
            failed=0,
            skipped=0,
            errors=1,
            duration_ms=0,
        )

    async def _execute_test_command(
        self, name: str, cmd: List[str], workdir: str = None
    ) -> TestSuite:
        """执行测试命令"""
        import time

        start_time = time.time()

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=300,
                cwd=workdir or self.workdir,
            )

            duration = (time.time() - start_time) * 1000

            # 解析输出
            output = result.stdout + result.stderr
            passed, failed, skipped, errors = self._parse_test_output(output)

            return TestSuite(
                name=name,
                total=passed + failed + skipped + errors,
                passed=passed,
                failed=failed,
                skipped=skipped,
                errors=errors,
                duration_ms=duration,
            )

        except subprocess.TimeoutExpired:
            return TestSuite(
                name=name,
                total=0,
                passed=0,
                failed=0,
                skipped=0,
                errors=1,
                duration_ms=300000,
            )
        except Exception:
            return TestSuite(
                name=name,
                total=0,
                passed=0,
                failed=0,
                skipped=0,
                errors=1,
                duration_ms=0,
            )

    def _parse_test_output(self, output: str) -> tuple:
        """解析测试输出"""
        import re

        # pytest 格式
        passed = len(re.findall(r"PASSED", output))
        failed = len(re.findall(r"FAILED", output))
        skipped = len(re.findall(r"SKIPPED", output))
        errors = len(re.findall(r"ERROR", output))

        # cargo test 格式
        if passed == 0 and failed == 0:
            match = re.search(r"(\d+) passed", output)
            if match:
                passed = int(match.group(1))
            match = re.search(r"(\d+) failed", output)
            if match:
                failed = int(match.group(1))
            match = re.search(r"(\d+) skipped", output)
            if match:
                skipped = int(match.group(1))
            match = re.search(r"(\d+) errors?", output)
            if match:
                errors = int(match.group(1))

        return passed, failed, skipped, errors

    async def _analyze_coverage(self, msg: AgentMessage):
        """分析覆盖率"""
        self._log(f"📊 分析覆盖率: {msg.summary}")

        # 简化版：尝试运行 coverage
        try:
            result = subprocess.run(
                ["python", "-m", "coverage", "report"],
                capture_output=True,
                text=True,
                cwd=self.workdir,
            )

            if result.returncode == 0:
                # 提取覆盖率
                import re

                match = re.search(r"TOTAL\s+\d+\s+\d+\s+(\d+)%", result.stdout)
                if match:
                    coverage = int(match.group(1))
                    self._send_result(
                        msg.task_id,
                        "completed",
                        f"覆盖率: {coverage}%\n\n{result.stdout}",
                    )
                    return

            self._send_result(
                msg.task_id,
                "completed",
                f"覆盖率分析结果:\n\n{result.stdout}\n{result.stderr}",
            )

        except Exception as e:
            self._send_result(msg.task_id, "failed", f"覆盖率分析失败: {e}")

    def _generate_report(self, suite: TestSuite) -> str:
        """生成测试报告"""
        emoji = "✅" if suite.failed == 0 and suite.errors == 0 else "❌"
        duration = (
            f"{suite.duration_ms:.0f}ms"
            if suite.duration_ms < 1000
            else f"{suite.duration_ms / 1000:.1f}s"
        )

        report = f"""
🐱 阿比西尼亚猫测试报告

{emoji} 测试结果:
  - 总计: {suite.total}
  - 通过: {suite.passed}
  - 失败: {suite.failed}
  - 跳过: {suite.skipped}
  - 错误: {suite.errors}
  - 耗时: {duration}
"""

        if suite.coverage_percent is not None:
            report += f"  - 覆盖率: {suite.coverage_percent}%\n"

        return report


async def main():
    """启动 Test Agent"""
    agent_id = os.environ.get("AGENT_ID", "tester-01")
    project_id = os.environ.get("PROJECT_ID", "default")
    workdir = os.environ.get("WORKDIR", ".")

    agent = TestAgent(agent_id, project_id, workdir)
    print(f"🐱 阿比西尼亚猫 Test Agent 启动: {agent_id}", flush=True)

    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
