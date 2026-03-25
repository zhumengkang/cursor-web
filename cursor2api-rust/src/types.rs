use serde::{Deserialize, Serialize};

// ==================== 配置类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThinkingConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompressionConfig {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolsConfig {
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_path: Option<String>,
}

/// 对应 TypeScript HotConfig
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HotConfig {
    pub cursor_model: Option<String>,
    pub timeout: Option<u64>,
    pub max_auto_continue: Option<u32>,
    pub max_history_messages: Option<i64>,
    pub max_history_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<CompressionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsConfig>,
    pub sanitize_response: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingConfig>,
    // 仅内部使用，不在 HotConfig 接口中暴露给前端
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_tokens: Option<Vec<String>>,
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub config: HotConfig,
    pub changes: Vec<String>,
}

// ==================== 日志类型 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTiming {
    pub phase: String,
    pub label: String,
    #[serde(rename = "startTime")]
    pub start_time: u64,
    #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestSummary {
    #[serde(rename = "requestId")]
    pub request_id: String,
    #[serde(rename = "startTime")]
    pub start_time: u64,
    #[serde(rename = "endTime", skip_serializing_if = "Option::is_none")]
    pub end_time: Option<u64>,
    pub method: Option<String>,
    pub path: Option<String>,
    pub model: Option<String>,
    pub stream: Option<bool>,
    #[serde(rename = "apiFormat", skip_serializing_if = "Option::is_none")]
    pub api_format: Option<String>,
    #[serde(rename = "hasTools", skip_serializing_if = "Option::is_none")]
    pub has_tools: Option<bool>,
    #[serde(rename = "toolCount", skip_serializing_if = "Option::is_none")]
    pub tool_count: Option<u32>,
    #[serde(rename = "messageCount", skip_serializing_if = "Option::is_none")]
    pub message_count: Option<u32>,
    pub status: Option<String>,
    #[serde(rename = "responseChars", skip_serializing_if = "Option::is_none")]
    pub response_chars: Option<u64>,
    #[serde(rename = "retryCount", skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<u32>,
    #[serde(rename = "continuationCount", skip_serializing_if = "Option::is_none")]
    pub continuation_count: Option<u32>,
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(rename = "statusReason", skip_serializing_if = "Option::is_none")]
    pub status_reason: Option<String>,
    #[serde(rename = "issueTags", skip_serializing_if = "Option::is_none")]
    pub issue_tags: Option<Vec<String>>,
    #[serde(rename = "toolCallsDetected", skip_serializing_if = "Option::is_none")]
    pub tool_calls_detected: Option<u32>,
    pub ttft: Option<u64>,
    #[serde(rename = "cursorApiTime", skip_serializing_if = "Option::is_none")]
    pub cursor_api_time: Option<u64>,
    #[serde(rename = "phaseTimings", skip_serializing_if = "Option::is_none")]
    pub phase_timings: Option<Vec<PhaseTiming>>,
    #[serde(rename = "thinkingChars", skip_serializing_if = "Option::is_none")]
    pub thinking_chars: Option<u64>,
    #[serde(rename = "systemPromptLength", skip_serializing_if = "Option::is_none")]
    pub system_prompt_length: Option<u64>,
    #[serde(rename = "inputTokens", skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(rename = "outputTokens", skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "authToken", skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub timestamp: u64,
    pub level: String,
    pub source: String,
    pub phase: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    #[serde(rename = "totalRequests")]
    pub total_requests: u64,
    #[serde(rename = "successCount")]
    pub success_count: u64,
    #[serde(rename = "degradedCount")]
    pub degraded_count: u64,
    #[serde(rename = "errorCount")]
    pub error_count: u64,
    #[serde(rename = "avgResponseTime")]
    pub avg_response_time: f64,
    #[serde(rename = "avgTTFT")]
    pub avg_ttft: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestsPage {
    pub requests: Vec<RequestSummary>,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

// ==================== API Keys ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub key_preview: String,
    pub note: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub last_used: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateKeyRequest {
    pub name: String,
    pub key_value: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateKeyRequest {
    pub name: String,
    pub note: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelBreakdown {
    pub model: String,
    pub count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DailyBreakdown {
    pub date: String,
    pub count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct KeyStats {
    pub total_requests: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub degraded_count: u64,
    pub success_rate: f64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_tokens: u64,
    pub avg_input_tokens: f64,
    pub avg_output_tokens: f64,
    pub avg_response_time: f64,
    pub models_breakdown: Vec<ModelBreakdown>,
    pub period_breakdown: Vec<DailyBreakdown>,
    pub granularity: String,
}

// ==================== 应用状态 ====================

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub config_path: String,
    pub db_path: String,
    pub log_dir: String,
    pub auth_tokens: Vec<String>,
    pub port: u16,
}
