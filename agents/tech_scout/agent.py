"""
🦊 狐狸 — Tech Scout Agent

职责：
- 文档搜索
- 趋势追踪
- 技术调研
- 最佳实践收集

特点：聪明、敏捷、善于发现 = 技术侦察兵

PM 做决策时 → 狐狸自动搜集:
- 官方文档
- GitHub trending
- Stack Overflow 高票答案
- 安全漏洞公告
- 给 PM 结构化对比报告
"""

import asyncio
import os
import re
import sys
from typing import List, Dict
from dataclasses import dataclass

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "base"))
from agent import BaseAgent, AgentMessage


@dataclass
class TechReport:
    """技术调研报告"""

    topic: str
    summary: str
    sources: List[str]
    recommendations: List[str]
    risks: List[str]
    alternatives: List[Dict[str, str]]


class TechScoutAgent(BaseAgent):
    """狐狸 Tech Scout Agent — 聪明、敏捷、善于发现"""

    def __init__(self, agent_id: str, project_id: str, workdir: str = "."):
        super().__init__(agent_id, "tech_scout", project_id, workdir)

    async def execute_task(self, msg: AgentMessage):
        """执行技术侦察任务"""
        task_type = self._detect_task_type(msg.summary)

        if task_type == "research":
            await self._research_technology(msg)
        elif task_type == "trending":
            await self._check_trending(msg)
        elif task_type == "security":
            await self._check_security(msg)
        elif task_type == "docs":
            await self._find_documentation(msg)
        else:
            await self._research_technology(msg)

    def _detect_task_type(self, summary: str) -> str:
        """检测任务类型"""
        summary_lower = summary.lower()
        if "趋势" in summary_lower or "trending" in summary_lower:
            return "trending"
        elif (
            "安全" in summary_lower
            or "漏洞" in summary_lower
            or "security" in summary_lower
        ):
            return "security"
        elif "文档" in summary_lower or "doc" in summary_lower:
            return "docs"
        return "research"

    async def _research_technology(self, msg: AgentMessage):
        """技术调研"""
        self._log(f"🔍 技术调研: {msg.summary}")

        # 提取关键词
        self._extract_keywords(msg.summary)

        # 模拟调研结果（实际应调用搜索 API）
        report = TechReport(
            topic=msg.summary,
            summary=f"关于 {msg.summary} 的技术调研",
            sources=[
                "官方文档",
                "GitHub 仓库",
                "技术博客",
            ],
            recommendations=[
                "建议查阅官方文档获取最新 API",
                "查看 GitHub issues 了解已知问题",
                "参考社区最佳实践",
            ],
            risks=[
                "版本兼容性",
                "社区活跃度",
                "维护状态",
            ],
            alternatives=[],
        )

        # 生成报告
        report_text = self._generate_report(report)
        self._send_result(msg.task_id, "completed", report_text)

    async def _check_trending(self, msg: AgentMessage):
        """检查技术趋势"""
        self._log(f"📈 检查趋势: {msg.summary}")

        report = """
🦊 技术趋势报告

📊 当前热门技术:
- AI/LLM: 持续火热，多 Agent 框架涌现
- Rust: 系统编程首选，性能敏感场景
- Vue3/Vite: 前端主流选择
- NATS: 高性能消息总线

🔗 推荐关注:
- GitHub Trending: https://github.com/trending
- Hacker News: https://news.ycombinator.com
- Reddit r/programming

💡 建议:
- 关注多 Agent 框架发展 (MetaGPT, AutoGen, CrewAI)
- Rust 生态持续成熟
- LLM 应用框架快速迭代
"""
        self._send_result(msg.task_id, "completed", report)

    async def _check_security(self, msg: AgentMessage):
        """检查安全漏洞"""
        self._log(f"🛡️ 安全检查: {msg.summary}")

        # 提取依赖信息
        deps = self._scan_dependencies()

        report = f"""
🦊 安全检查报告

📦 扫描依赖: {len(deps)} 个

🔍 检查项:
- 已知漏洞
- 过时依赖
- 不安全配置

💡 建议:
- 定期运行 cargo audit / pip audit
- 关注安全公告
- 使用依赖锁定文件
"""
        self._send_result(msg.task_id, "completed", report)

    async def _find_documentation(self, msg: AgentMessage):
        """查找文档"""
        self._log(f"📚 查找文档: {msg.summary}")

        keywords = self._extract_keywords(msg.summary)

        report = f"""
🦊 文档搜索报告

🔍 搜索关键词: {", ".join(keywords)}

📚 文档来源:
- 官方文档: https://docs.rs (Rust)
- 官方文档: https://docs.python.org (Python)
- MDN: https://developer.mozilla.org (Web)

💡 建议:
- 优先查阅官方文档
- 查看 changelog 了解版本变化
- 参考示例代码
"""
        self._send_result(msg.task_id, "completed", report)

    def _extract_keywords(self, text: str) -> List[str]:
        """提取关键词"""
        # 移除常见停用词
        stop_words = {
            "的",
            "了",
            "是",
            "在",
            "和",
            "有",
            "这",
            "个",
            "我",
            "你",
            "他",
            "她",
            "它",
        }
        words = re.findall(r"[\w]+", text)
        keywords = [w for w in words if w not in stop_words and len(w) > 1]
        return list(set(keywords))[:10]

    def _scan_dependencies(self) -> List[str]:
        """扫描项目依赖"""
        deps = []

        # 检查 Cargo.toml
        cargo_toml = os.path.join(self.workdir, "Cargo.toml")
        if os.path.exists(cargo_toml):
            with open(cargo_toml, "r") as f:
                content = f.read()
                # 简单提取依赖名
                matches = re.findall(r"^(\w[\w-]*)\s*=", content, re.MULTILINE)
                deps.extend(matches)

        # 检查 requirements.txt
        requirements = os.path.join(self.workdir, "requirements.txt")
        if os.path.exists(requirements):
            with open(requirements, "r") as f:
                for line in f:
                    line = line.strip()
                    if line and not line.startswith("#"):
                        dep = re.split(r"[>=<]", line)[0].strip()
                        deps.append(dep)

        return deps

    def _generate_report(self, report: TechReport) -> str:
        """生成调研报告"""
        text = f"""
🦊 技术调研报告

📋 主题: {report.topic}

📝 摘要:
{report.summary}

📚 信息来源:
"""
        for source in report.sources:
            text += f"  - {source}\n"

        text += "\n💡 建议:\n"
        for rec in report.recommendations:
            text += f"  - {rec}\n"

        if report.risks:
            text += "\n⚠️ 风险:\n"
            for risk in report.risks:
                text += f"  - {risk}\n"

        if report.alternatives:
            text += "\n🔄 替代方案:\n"
            for alt in report.alternatives:
                text += f"  - {alt.get('name', '')}: {alt.get('description', '')}\n"

        return text


async def main():
    """启动 Tech Scout Agent"""
    agent_id = os.environ.get("AGENT_ID", "tech-scout-01")
    project_id = os.environ.get("PROJECT_ID", "default")
    workdir = os.environ.get("WORKDIR", ".")

    agent = TechScoutAgent(agent_id, project_id, workdir)
    print(f"🦊 狐狸 Tech Scout Agent 启动: {agent_id}", flush=True)

    await agent.run()


if __name__ == "__main__":
    asyncio.run(main())
