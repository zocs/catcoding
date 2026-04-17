# Rust 测试布局说明

计划书（附录 D）把测试放在 `tests/rust/test_{watchdog,scheduler,state,xp}.rs`。
当前实现采用 Cargo 原生的 `#[cfg(test)]` 内联单元测试：

| 原计划文件 | 实际位置 |
|---|---|
| test_watchdog.rs | `daemon/src/watchdog.rs` 底部 `#[cfg(test)] mod tests` |
| test_scheduler.rs | `daemon/src/scheduler.rs` 底部 `#[cfg(test)] mod tests` |
| test_state.rs | `daemon/src/state.rs` 底部 `#[cfg(test)] mod tests` |
| test_xp.rs | `daemon/src/xp.rs` 底部 `#[cfg(test)] mod tests` |
| test_permission | `daemon/src/permission.rs` 底部 `#[cfg(test)] mod tests`（既有） |

## 为什么不放在 `tests/rust/`

Cargo 集成测试（`tests/` 目录下的独立 crate）只能访问 **public** API。
`daemon` 当前只有 `[[bin]]` 没有 `[lib]`，内部模块 (`scheduler::Scheduler` 等) 对
外不可见。把测试挪到根目录 `tests/rust/` 需要先把 daemon 改造成 lib+bin 双目标，
并把每个要测的项逐一 `pub`。这超出本次"Quick+Medium"档范围，留到下一次会话。

## 如何跑

```bash
cd ~/Devs/catcoding
cargo test --workspace
```

下一次会话会做：
1. 把 `daemon` 拆为 `[lib] name = "catcoding"` + `[[bin]] name = "catcoding-daemon"`
2. 所有 `mod X;` 移到 `daemon/src/lib.rs` 并 `pub mod X;`
3. 把内联测试移到 `daemon/tests/*.rs`（标准 Cargo 集成测试布局）
4. 根 `tests/rust/` 改成工作区成员 `integration-tests` 做端到端（HTTP + NATS）测试
