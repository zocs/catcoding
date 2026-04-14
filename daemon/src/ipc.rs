use anyhow::Result;

/// Agent IPC 协议
///
/// Daemon 与 Python Agent 之间的通信协议
/// 通信方式：stdin/stdout JSON-RPC 风格
pub struct IpcProtocol;

impl IpcProtocol {
    /// 发送任务给 Agent
    pub fn encode_task(task_id: &str, title: &str, description: &str) -> String {
        serde_json::json!({
            "method": "task.assign",
            "params": {
                "task_id": task_id,
                "title": title,
                "description": description,
            }
        })
        .to_string()
    }

    /// 解码 Agent 输出
    pub fn decode_output(line: &str) -> Result<serde_json::Value> {
        let value: serde_json::Value = serde_json::from_str(line)?;
        Ok(value)
    }

    /// 解码心跳
    pub fn decode_heartbeat(line: &str) -> Result<Option<String>> {
        let value: serde_json::Value = serde_json::from_str(line)?;
        if value["method"] == "heartbeat" {
            Ok(value["agent_id"].as_str().map(String::from))
        } else {
            Ok(None)
        }
    }
}
