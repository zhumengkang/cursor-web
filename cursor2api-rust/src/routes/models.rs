use axum::Json;
use serde_json::{json, Value};

/// Cursor 官网已知可用模型列表（按提供商分组）
/// 来源：Cursor 设置页 > Models，定期手动更新
pub async fn get_models() -> Json<Value> {
    Json(json!({
        "models": [
            // Anthropic
            { "id": "anthropic/claude-opus-4-5",           "label": "Claude Opus 4.5",           "provider": "Anthropic" },
            { "id": "anthropic/claude-sonnet-4-5",         "label": "Claude Sonnet 4.5",         "provider": "Anthropic" },
            { "id": "anthropic/claude-haiku-4-5",          "label": "Claude Haiku 4.5",          "provider": "Anthropic" },
            { "id": "anthropic/claude-opus-4-6",           "label": "Claude Opus 4.6",           "provider": "Anthropic" },
            { "id": "anthropic/claude-sonnet-4-6",         "label": "Claude Sonnet 4.6",         "provider": "Anthropic" },
            { "id": "anthropic/claude-3-7-sonnet",         "label": "Claude 3.7 Sonnet",         "provider": "Anthropic" },
            { "id": "anthropic/claude-3-5-sonnet",         "label": "Claude 3.5 Sonnet",         "provider": "Anthropic" },
            { "id": "anthropic/claude-3-5-haiku",          "label": "Claude 3.5 Haiku",          "provider": "Anthropic" },
            { "id": "anthropic/claude-3-opus",             "label": "Claude 3 Opus",             "provider": "Anthropic" },
            // OpenAI
            { "id": "openai/gpt-4o",                      "label": "GPT-4o",                    "provider": "OpenAI" },
            { "id": "openai/gpt-4o-mini",                 "label": "GPT-4o mini",               "provider": "OpenAI" },
            { "id": "openai/gpt-4.1",                     "label": "GPT-4.1",                   "provider": "OpenAI" },
            { "id": "openai/gpt-4.1-mini",                "label": "GPT-4.1 mini",              "provider": "OpenAI" },
            { "id": "openai/gpt-4.1-nano",                "label": "GPT-4.1 nano",              "provider": "OpenAI" },
            { "id": "openai/gpt-5.1-codex-mini",          "label": "GPT-5.1 Codex Mini",       "provider": "OpenAI" },
            { "id": "openai/o3",                          "label": "o3",                        "provider": "OpenAI" },
            { "id": "openai/o4-mini",                     "label": "o4-mini",                   "provider": "OpenAI" },
            // Google
            { "id": "google/gemini-2.5-pro",              "label": "Gemini 2.5 Pro",            "provider": "Google" },
            { "id": "google/gemini-2.5-flash",            "label": "Gemini 2.5 Flash",          "provider": "Google" },
            { "id": "google/gemini-2.0-flash",            "label": "Gemini 2.0 Flash",          "provider": "Google" },
            { "id": "google/gemini-3-flash",              "label": "Gemini 3 Flash",            "provider": "Google" },
            // xAI
            { "id": "xai/grok-3",                         "label": "Grok 3",                    "provider": "xAI" },
            { "id": "xai/grok-3-mini",                    "label": "Grok 3 Mini",               "provider": "xAI" },
            // DeepSeek
            { "id": "deepseek/deepseek-r2",               "label": "DeepSeek R2",               "provider": "DeepSeek" },
            { "id": "deepseek/deepseek-v3",               "label": "DeepSeek V3",               "provider": "DeepSeek" },
        ]
    }))
}
