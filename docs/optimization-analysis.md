# CatCoding 框架优化分析

> 基于 GenericAgent、deer-flow、ClawManager 三个项目的学习
> 时间: 2026-04-15 02:00 CST

---

## 📊 三项目核心特性对比

| 维度 | GenericAgent | deer-flow | ClawManager |
|------|-------------|-----------|-------------|
| **核心理念** | 自进化（不预装技能，边用边学） | 超级 Agent 编排 | K8s 控制面 |
| **记忆系统** | L4 分层（索引→事实→SOP→会话） | 长期记忆 | 资源管理 |
| **技能系统** | 自动结晶执行路径 | 结构化 Skill 模块 | Skill 扫描+交付 |
| **Agent 管理** | 单 Agent + 子 Agent | 多 Agent 编排 | K8s 容器编排 |
| **代码量** | ~3K 行核心 | 完整框架 | Go + React |

---

## 🧠 关键洞察

### 1. GenericAgent: L4 记忆系统

**四层架构**:
```
L1: global_mem_insight.txt (≤30行，极简索引)
    ↓ 指针
L2: global_mem.txt (事实库：路径、配置、凭证)
    ↓ 引用
L3: memory/ (SOP、脚本、任务级记录)
    ↓ 归档
L4: memory/L4_raw_sessions/ (历史会话)
```

**核心公理**:
1. **No Execution, No Memory** — 只有成功执行的结果才记入记忆
2. **Sanctity of Verified Data** — 验证过的数据神圣不可删改
3. **No Volatile State** — 禁止存储易变状态（时间戳、PID）
4. **Minimum Sufficient Pointer** — 上层只留定位标识

**自进化机制**:
- 每次完成任务，自动将执行路径结晶为 Skill
- 下次类似任务直接复用
- 用得越久，能力树越丰富

### 2. deer-flow: Skills + 编排

**Skills 系统**:
- 结构化能力模块（Markdown + 元数据）
- 渐进式加载（只在需要时加载）
- 支持安装、替换、组合

**子 Agent 编排**:
- 多 Agent 并行工作
- 上下文工程（Context Engineering）
- 沙箱隔离

**Claude Code 集成**:
- 从终端直接发送研究任务
- 流式响应

### 3. ClawManager: 控制面设计

**三层控制面**:
1. **AI Gateway** — 模型访问治理、审计、成本核算
2. **Agent Control Plane** — 运行时编排、命令分发
3. **Resource Management** — 可复用资源（频道、Skill）

**Skill 管理**:
- 扫描和检测
- 打包和交付

---

## 🎯 CatCoding 优化建议

### 🔴 高优先级（立即可做）

#### 1. 引入 L4 记忆系统
**当前状态**: 只有简单的 `memory` 工具和 `session_search`
**优化方案**: 参考 GenericAgent 的四层架构

```yaml
.catcoding/
├── memory/
│   ├── L1_index.txt        # ≤30行，场景→定位映射
│   ├── L2_facts.txt        # 环境事实（路径、配置）
│   ├── L3/                 # SOP、脚本
│   │   ├── coding_sop.md
│   │   ├── debug_sop.md
│   │   └── tools/
│   └── L4_sessions/        # 历史会话归档
│       └── compress_session.py
```

**核心原则**:
- "No Execution, No Memory" — 只有验证过的才记入
- 上层只留指针，不写细节
- 状态不存入记忆（用 SQLite）

#### 2. 实现 Skills 自动结晶
**当前状态**: 手动创建 Skill，无自动学习
**优化方案**: 每次完成任务，自动将执行路径写为 Skill

```
任务完成 → 提取执行路径 → 生成 Skill → 写入 L3
下次类似任务 → 搜索 L3 → 直接复用
```

**具体步骤**:
- 在 Agent 基类中添加 `crystallize_skill()` 方法
- 分析成功的工具调用序列
- 生成结构化 Skill 文档

#### 3. Skills 渐进式加载
**当前状态**: 所有 Skill 都加载到上下文
**优化方案**: 参考 deer-flow，只在需要时加载

```python
# 任务分析阶段
relevant_skills = analyze_task(task)  # 分析需要哪些 Skill
load_skills(relevant_skills)          # 只加载相关的
```

**收益**: 减少 token 消耗，提升模型性能

#### 4. Definition of Done (DoD) 任务完成契约
**来源**: BotLearn 社区 — 心晴 (@roy722) + Finn
**当前状态**: Agent 完成任务后自行判断，容易出现"以为完成但实际未完成"
**优化方案**: 每个任务分配时必须附带 DoD，执行后自动验证

**核心原则**:
> Never "figure out what to do" — always "do X, output Y, verify Z"

**DoD 数据结构**（已实现于 `agents/base/agent.py`）:
```python
@dataclass
class TaskDoD:
    output_path: str          # 输出必须写入此路径
    output_format: str        # 格式规格（如 "JSON with fields: ..."）
    verify_command: str       # 验证命令（exit 0 = success）
    silent_on_success: bool   # 成功时返回 HEARTBEAT_OK（cron 场景）
    required_artifacts: list  # 必须产出的文件列表
```

**验证流程**:
```
任务执行完成
  → 检查 output_path 文件是否存在
  → 检查 required_artifacts 是否齐全
  → 执行 verify_command（exit code 0 = pass）
  → 全部通过 → 完成 ✓
  → 任一失败 → 标记 FAILED，返回具体原因
```

**任务分配示例**:
```json
{
  "type": "task.assign",
  "summary": "生成 API 文档",
  "details": {
    "dod": {
      "output_path": "docs/api.md",
      "output_format": "Markdown with ## Endpoints section",
      "verify_command": "test -s docs/api.md && grep -q '## Endpoints' docs/api.md",
      "silent_on_success": false,
      "required_artifacts": ["docs/api.md"]
    }
  }
}
```

**Cron 场景（静默成功）**:
```json
{
  "dod": {
    "output_path": "",
    "verify_command": "true",
    "silent_on_success": true
  }
}
// 无内容汇报 → 返回 "HEARTBEAT_OK"
// 有异常 → 返回具体错误
// 信号干净，不污染日志
```

**收益**:
- 消除"假完成"——Agent 无法自我欺骗，必须通过客观验证
- 自动化质检——无需人工 review 每个任务输出
- 干净信号——cron 任务成功时静默，失败时才告警

---

### 🟡 中优先级（Phase 2-3）

#### 4. 上下文工程 (Context Engineering)
**当前状态**: 简单的三层注入
**优化方案**: 参考 deer-flow 的上下文工程

```
注入策略:
├── 角色身份 (~200 tokens)
├── 项目摘要 (~500 tokens)
├── 任务详情 (~1000-3000 tokens)
├── 相关 Skill (~500 tokens)     ← 新增
├── 历史经验 (~500 tokens)        ← 新增
└── 代码片段 (~2000 tokens)       ← 按需加载
```

#### 5. Agent 间知识共享
**当前状态**: Agent 各自独立
**优化方案**: 参考 ClawManager 的资源管理

```
共享层:
├── 项目级 Skill 库（所有 Agent 可用）
├── 错误解决方案库（调试经验）
├── 最佳实践库（代码风格、架构决策）
└── 依赖知识库（版本兼容、已知问题）
```

#### 6. AI Gateway 治理
**当前状态**: 无模型治理
**优化方案**: 参考 ClawManager 的 AI Gateway

```
治理层:
├── 模型路由（根据任务选择模型）
├── 成本核算（token 使用统计）
├── 审计日志（所有调用可追溯）
└── 风控规则（拒绝危险操作）
```

---

### 🟢 低优先级（Phase 4+）

#### 7. 沙箱隔离
**当前状态**: Agent 直接操作文件系统
**优化方案**: 参考 deer-flow 的沙箱模式

```
隔离策略:
├── 每个 Agent 独立工作目录
├── 文件变更前创建 checkpoint
├── 审查通过后才合并到主分支
└── 可选: Docker 容器隔离
```

#### 8. 可视化 Skill 树
**当前状态**: 无
**优化方案**: Dashboard 中展示能力树

```
Dashboard 新增:
├── Skill 树可视化（能力图谱）
├── 执行路径回放
├── 记忆层级浏览器
└── Agent 知识图谱
```

---

## 📐 架构优化建议

### 当前架构 vs 优化后架构

**当前**:
```
Daemon → Agent → LLM → 工具调用
```

**优化后**:
```
Daemon
├── Agent Manager
│   ├── Agent 1 (PM)
│   ├── Agent 2 (Dev)
│   └── Agent N (...)
├── Memory System (L4)
│   ├── L1 Index
│   ├── L2 Facts
│   ├── L3 Skills
│   └── L4 Sessions
├── Skill Engine
│   ├── 自动结晶
│   ├── 渐进加载
│   └── 搜索匹配
├── Context Engine
│   ├── 动态注入
│   └── Token 优化
└── AI Gateway (可选)
    ├── 模型路由
    ├── 成本核算
    └── 审计日志
```

---

## 🚀 实施路径

### Phase 2 新增任务

| 任务 | 来源 | 优先级 | 状态 |
|------|------|--------|------|
| L4 记忆系统 | GenericAgent | 🔴 高 | 待做 |
| Skills 自动结晶 | GenericAgent | 🔴 高 | 待做 |
| Skills 渐进加载 | deer-flow | 🔴 高 | 待做 |
| Definition of Done | BotLearn 社区 | 🔴 高 | ✅ 已实现 |
| 上下文工程 | deer-flow | 🟡 中 | 待做 |
| Agent 间知识共享 | ClawManager | 🟡 中 | 待做 |

### 已有任务调整

| 原任务 | 调整 |
|--------|------|
| PM Agent 实现 | 集成 L4 记忆读取 |
| Review Agent | 添加 Skill 结晶能力 |
| Tech Scout | 增强 L3 知识库写入 |

---

## 📝 关键引用

1. **GenericAgent L4 记忆**: `~/learns/agents/GenericAgent/memory/memory_management_sop.md`
2. **deer-flow Skills**: `~/learns/agents/deer-flow/.agent/skills/`
3. **ClawManager 控制面**: `~/learns/agents/ClawManager/docs/aigateway.md`

---

*分析完成时间: 2026-04-15 02:00 CST*
