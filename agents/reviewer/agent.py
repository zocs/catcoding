"""
🐱 玄猫 — Code Review Agent

职责：
- 代码审查
- 找 bug（抓老鼠）
- 质量把关
- 安全检查

Bug 分级（老鼠系统）：
- 🐭 小老鼠 — 简单 bug（拼写错误、语法问题）
- 🐀 大老鼠 — 顽固 bug（逻辑错误、边界情况）
- 🦇 蝙蝠 — 异常刁钻的 bug（时序问题、竞态条件）
- 🐉 恶龙 — 架构级 bug（系统性问题，需要重构）
"""

import asyncio
import os
import re
import sys
from typing import List
from dataclasses import dataclass
from enum import Enum

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent, AgentMessage


class BugSeverity(Enum):
    """Bug 严重程度"""

    MOUSE = "mouse"  # 🐭 小老鼠
    RAT = "rat"  # 🐀 大老鼠
    BAT = "bat"  # 🦇 蝙蝠
    DRAGON = "dragon"  # 🐉 恶龙


@dataclass
class Bug:
    """发现的 Bug"""

    file: str
    line: int
    severity: BugSeverity
    category: str
    description: str
    suggestion: str


@dataclass
class ReviewResult:
    """审查结果"""

    file_path: str
    total_lines: int
    bugs_found: List[Bug]
    quality_score: float  # 0-100
    passed: bool
    summary: str


class ReviewAgent(BaseAgent):
    """玄猫 Code Review Agent — 神秘、敏锐、找 bug"""

    # 代码质量规则
    RULES = {
        # 🐭 小老鼠
        "trailing_whitespace": {
            "severity": BugSeverity.MOUSE,
            "pattern": r"[ \t]+$",
            "message": "行尾有多余空格",
        },
        "todo_fixme": {
            "severity": BugSeverity.MOUSE,
            "pattern": r"#\s*(TODO|FIXME|HACK|XXX)",
            "message": "存在 TODO/FIXME 标记",
        },
        "hardcoded_path": {
            "severity": BugSeverity.MOUSE,
            "pattern": r"""['"](\/[a-zA-Z]|C:\\|D:\\)""",
            "message": "硬编码路径",
        },
        # 🐀 大老鼠
        "bare_except": {
            "severity": BugSeverity.RAT,
            "pattern": r"except\s*:",
            "message": "裸 except 捕获所有异常（应指定异常类型）",
        },
        "magic_number": {
            "severity": BugSeverity.RAT,
            "pattern": r"(?<![a-zA-Z_])\d{3,}(?![a-zA-Z_])",
            "message": "魔法数字（应定义为常量）",
        },
        "long_function": {
            "severity": BugSeverity.RAT,
            "pattern": None,  # 特殊检测
            "message": "函数过长（超过50行）",
        },
        # 🦇 蝙蝠
        "global_state": {
            "severity": BugSeverity.BAT,
            "pattern": r"^(global|mutable)\s+",
            "message": "全局可变状态（可能导致竞态）",
        },
        "nested_callback": {
            "severity": BugSeverity.BAT,
            "pattern": r"callback.*callback",
            "message": "嵌套回调（回调地狱）",
        },
    }

    def __init__(self, agent_id: str, project_id: str, workdir: str = "."):
        super().__init__(agent_id, "reviewer", project_id, workdir)

    async def execute_task(self, msg: AgentMessage):
        """执行代码审查"""
        self._log(f"🔍 开始审查: {msg.summary}")

        # 获取要审查的文件
        files = msg.artifacts if msg.artifacts else []

        if not files:
            # 尝试从 summary 中提取文件路径
            files = self._extract_files_from_summary(msg.summary)

        if not files:
            self._send_result(msg.task_id, "failed", "未指定要审查的文件")
            return

        # 审查每个文件
        all_results = []
        for file_path in files:
            if os.path.exists(file_path):
                result = self._review_file(file_path)
                all_results.append(result)
                self._log(f"🔍 {file_path}: {len(result.bugs_found)} 个问题")
            else:
                self._log(f"⚠️ 文件不存在: {file_path}")

        # 生成审查报告
        report = self._generate_report(all_results)

        # 判断是否通过
        all_passed = all(r.passed for r in all_results)
        status = "completed" if all_passed else "failed"

        self._send_result(msg.task_id, status, report)

    def _extract_files_from_summary(self, summary: str) -> List[str]:
        """从摘要中提取文件路径"""
        # 匹配常见文件路径模式
        patterns = [
            r"[a-zA-Z0-9_/\\\-]+\.(py|rs|js|ts|vue|json|yaml|yml|toml)",
            r"src/[a-zA-Z0-9_/\\\-]+",
        ]
        files = []
        for pattern in patterns:
            matches = re.findall(pattern, summary)
            files.extend(matches)
        return list(set(files))

    def _review_file(self, file_path: str) -> ReviewResult:
        """审查单个文件"""
        with open(file_path, "r", encoding="utf-8") as f:
            content = f.read()
            lines = content.split("\n")

        bugs = []

        # 应用规则检测
        for rule_name, rule in self.RULES.items():
            if rule["pattern"]:
                for i, line in enumerate(lines, 1):
                    if re.search(rule["pattern"], line):
                        bugs.append(
                            Bug(
                                file=file_path,
                                line=i,
                                severity=rule["severity"],
                                category=rule_name,
                                description=rule["message"],
                                suggestion=self._get_suggestion(rule_name, line),
                            )
                        )

        # 检测长函数
        bugs.extend(self._check_long_functions(file_path, lines))

        # 计算质量分数
        quality_score = self._calculate_quality_score(len(lines), bugs)

        # 判断是否通过
        critical_bugs = [
            b for b in bugs if b.severity in (BugSeverity.BAT, BugSeverity.DRAGON)
        ]
        passed = len(critical_bugs) == 0 and quality_score >= 60

        # 生成摘要
        summary = self._generate_file_summary(file_path, bugs, quality_score, passed)

        return ReviewResult(
            file_path=file_path,
            total_lines=len(lines),
            bugs_found=bugs,
            quality_score=quality_score,
            passed=passed,
            summary=summary,
        )

    def _check_long_functions(self, file_path: str, lines: List[str]) -> List[Bug]:
        """检测过长的函数"""
        bugs = []
        func_start = None
        func_name = None

        for i, line in enumerate(lines, 1):
            # 检测函数定义
            match = re.match(r"^(\s*)def\s+(\w+)\s*\(", line)
            if match:
                if func_start and func_name:
                    length = i - func_start
                    if length > 50:
                        bugs.append(
                            Bug(
                                file=file_path,
                                line=func_start,
                                severity=BugSeverity.RAT,
                                category="long_function",
                                description=f"函数 {func_name} 过长 ({length} 行)",
                                suggestion=f"考虑将 {func_name} 拆分为更小的函数",
                            )
                        )
                func_start = i
                func_name = match.group(2)

        # 检查最后一个函数
        if func_start and func_name:
            length = len(lines) - func_start
            if length > 50:
                bugs.append(
                    Bug(
                        file=file_path,
                        line=func_start,
                        severity=BugSeverity.RAT,
                        category="long_function",
                        description=f"函数 {func_name} 过长 ({length} 行)",
                        suggestion=f"考虑将 {func_name} 拆分为更小的函数",
                    )
                )

        return bugs

    def _get_suggestion(self, rule_name: str, line: str) -> str:
        """获取修复建议"""
        suggestions = {
            "trailing_whitespace": "删除行尾空格",
            "todo_fixme": "解决或移除 TODO/FIXME",
            "hardcoded_path": "使用配置或环境变量",
            "bare_except": "指定异常类型，如 except ValueError:",
            "magic_number": "定义为常量，如 MAX_RETRIES = 3",
            "long_function": "拆分为更小的函数",
            "global_state": "使用依赖注入或局部变量",
            "nested_callback": "考虑使用 async/await",
        }
        return suggestions.get(rule_name, "请检查并修复")

    def _calculate_quality_score(self, total_lines: int, bugs: List[Bug]) -> float:
        """计算质量分数 (0-100)"""
        if total_lines == 0:
            return 100.0

        # 按严重程度扣分
        deductions = {
            BugSeverity.MOUSE: 0.5,
            BugSeverity.RAT: 2.0,
            BugSeverity.BAT: 5.0,
            BugSeverity.DRAGON: 20.0,
        }

        total_deduction = sum(deductions.get(b.severity, 1.0) for b in bugs)

        # 每100行允许的问题数
        normalized_deduction = total_deduction / (total_lines / 100)

        score = max(0, 100 - normalized_deduction)
        return round(score, 1)

    def _generate_file_summary(
        self, file_path: str, bugs: List[Bug], score: float, passed: bool
    ) -> str:
        """生成文件审查摘要"""
        emoji = "✅" if passed else "❌"
        mouse_count = sum(1 for b in bugs if b.severity == BugSeverity.MOUSE)
        rat_count = sum(1 for b in bugs if b.severity == BugSeverity.RAT)
        bat_count = sum(1 for b in bugs if b.severity == BugSeverity.BAT)
        dragon_count = sum(1 for b in bugs if b.severity == BugSeverity.DRAGON)

        return f"{emoji} {file_path} — 质量分: {score} — 🐭{mouse_count} 🐀{rat_count} 🦇{bat_count} 🐉{dragon_count}"

    def _generate_report(self, results: List[ReviewResult]) -> str:
        """生成审查报告"""
        total_files = len(results)
        passed_files = sum(1 for r in results if r.passed)
        total_bugs = sum(len(r.bugs_found) for r in results)
        avg_score = (
            sum(r.quality_score for r in results) / total_files
            if total_files > 0
            else 0
        )

        report = f"""
🐱 玄猫代码审查报告

📊 总体情况:
  - 审查文件: {total_files}
  - 通过: {passed_files}/{total_files}
  - 发现问题: {total_bugs}
  - 平均质量分: {avg_score:.1f}

📋 文件详情:
"""
        for result in results:
            report += f"  {result.summary}\n"

        # 列出严重问题
        serious_bugs = []
        for result in results:
            for bug in result.bugs_found:
                if bug.severity in (
                    BugSeverity.RAT,
                    BugSeverity.BAT,
                    BugSeverity.DRAGON,
                ):
                    serious_bugs.append(bug)

        if serious_bugs:
            report += "\n🔴 严重问题:\n"
            for bug in serious_bugs[:5]:  # 最多显示5个
                emoji = {"rat": "🐀", "bat": "🦇", "dragon": "🐉"}.get(
                    bug.severity.value, "❓"
                )
                report += f"  {emoji} {bug.file}:{bug.line} — {bug.description}\n"
                report += f"     💡 建议: {bug.suggestion}\n"

        return report


async def main():
    """启动 Review Agent"""
    agent_id = os.environ.get("AGENT_ID", "reviewer-01")
    project_id = os.environ.get("PROJECT_ID", "default")
    workdir = os.environ.get("WORKDIR", ".")

    agent = ReviewAgent(agent_id, project_id, workdir)
    print(f"🐱 玄猫 Review Agent 启动: {agent_id}", flush=True)

    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
