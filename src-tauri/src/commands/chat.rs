use chrono::Utc;
use tauri::ipc::Channel;
use tauri::State;
use uuid::Uuid;

use crate::ai::chat::{ChatEvent, ChatMessage, ChatRole, ChatTurn, ToolCall, ToolResult};
use crate::ai::error::{ProviderError, ProviderResult};
use crate::ai::metadata::find_metadata;
use crate::ai::prompts::CHAT_SYSTEM_PROMPT;
use crate::ai::provider::{build_provider, ProviderInputs};
use crate::ai::tools::{execute_tool, tool_catalog};
use crate::ai::{keychain, Provider};
use crate::db::models::{Message, MessageRole, ProviderConfig};
use crate::db::Database;

const MAX_TOOL_ITERATIONS: usize = 5;

#[tauri::command(rename_all = "camelCase")]
pub async fn chat(
    db: State<'_, Database>,
    provider_id: String,
    conversation_id: String,
    user_message: String,
    on_event: Channel<ChatEvent>,
) -> ProviderResult<Message> {
    let trimmed = user_message.trim();
    if trimmed.is_empty() {
        return Err(ProviderError::InvalidResponse("empty message".into()));
    }

    enforce_provider_lock(db.pool(), &conversation_id, &provider_id).await?;

    insert_message(
        db.pool(),
        &conversation_id,
        MessageRole::User,
        trimmed,
        None,
        None,
    )
    .await?;

    let history = load_messages(db.pool(), &conversation_id).await?;
    let mut chat_messages = history.into_iter().map(message_to_chat).collect::<Vec<_>>();

    let provider = resolve_provider(&db, &provider_id).await?;
    let tools = if provider.supports_tools() {
        tool_catalog()
    } else {
        Vec::new()
    };

    let _ = on_event.send(ChatEvent::Thinking);

    let mut final_text = String::new();

    for _ in 0..MAX_TOOL_ITERATIONS {
        let turn = provider
            .chat_turn(&chat_messages, &tools, CHAT_SYSTEM_PROMPT)
            .await?;

        let ChatTurn { text, tool_calls } = turn;

        if tool_calls.is_empty() {
            final_text = text;
            break;
        }

        chat_messages.push(ChatMessage {
            role: ChatRole::Assistant,
            content: text.clone(),
            tool_calls: tool_calls.clone(),
            tool_call_id: None,
        });

        let tool_calls_json = serde_json::to_string(&tool_calls)?;
        insert_message(
            db.pool(),
            &conversation_id,
            MessageRole::Assistant,
            &text,
            Some(&tool_calls_json),
            None,
        )
        .await?;

        for call in tool_calls {
            let _ = on_event.send(ChatEvent::ToolCall { call: call.clone() });

            let result_content = match execute_tool(db.pool(), &call.name, &call.arguments).await {
                Ok(s) => s,
                Err(err) => format!(r#"{{"error": "{}"}}"#, err.to_string().replace('"', "\\\"")),
            };

            let _ = on_event.send(ChatEvent::ToolResult {
                result: ToolResult {
                    tool_call_id: call.id.clone(),
                    content: result_content.clone(),
                },
            });

            chat_messages.push(ChatMessage {
                role: ChatRole::Tool,
                content: result_content.clone(),
                tool_calls: Vec::new(),
                tool_call_id: Some(call.id.clone()),
            });

            insert_message(
                db.pool(),
                &conversation_id,
                MessageRole::Tool,
                &result_content,
                None,
                Some(&call.id),
            )
            .await?;
        }
    }

    let assistant = insert_message(
        db.pool(),
        &conversation_id,
        MessageRole::Assistant,
        &final_text,
        None,
        None,
    )
    .await?;

    bump_conversation_timestamp(
        db.pool(),
        &conversation_id,
        &provider_id,
        provider.metadata().default_model,
    )
    .await;

    let _ = on_event.send(ChatEvent::Text {
        content: final_text,
    });
    let _ = on_event.send(ChatEvent::Done);

    Ok(assistant)
}

async fn insert_message(
    pool: &sqlx::SqlitePool,
    conversation_id: &str,
    role: MessageRole,
    content: &str,
    tool_calls: Option<&str>,
    tool_call_id: Option<&str>,
) -> ProviderResult<Message> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, tool_calls, tool_call_id, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .bind(tool_calls)
    .bind(tool_call_id)
    .bind(now)
    .execute(pool)
    .await?;

    let msg = sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await?;
    Ok(msg)
}

async fn load_messages(
    pool: &sqlx::SqlitePool,
    conversation_id: &str,
) -> ProviderResult<Vec<Message>> {
    sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at ASC",
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

async fn conversation_provider_lock(
    pool: &sqlx::SqlitePool,
    conversation_id: &str,
) -> ProviderResult<Option<String>> {
    let locked: Option<Option<String>> =
        sqlx::query_scalar("SELECT provider_id FROM conversations WHERE id = ?")
            .bind(conversation_id)
            .fetch_optional(pool)
            .await?;
    Ok(locked.flatten())
}

async fn enforce_provider_lock(
    pool: &sqlx::SqlitePool,
    conversation_id: &str,
    attempted_provider: &str,
) -> ProviderResult<()> {
    if let Some(locked_to) = conversation_provider_lock(pool, conversation_id).await? {
        if locked_to != attempted_provider {
            return Err(ProviderError::ConversationLocked {
                locked_to,
                attempted: attempted_provider.to_string(),
            });
        }
    }
    Ok(())
}

async fn bump_conversation_timestamp(
    pool: &sqlx::SqlitePool,
    conversation_id: &str,
    provider_id: &str,
    model: &str,
) {
    let now = Utc::now();
    let _ = sqlx::query(
        "UPDATE conversations
         SET last_message_at = ?,
             provider_id = COALESCE(provider_id, ?),
             model = COALESCE(model, ?)
         WHERE id = ?",
    )
    .bind(now)
    .bind(provider_id)
    .bind(model)
    .bind(conversation_id)
    .execute(pool)
    .await;
}

fn message_to_chat(msg: Message) -> ChatMessage {
    let role = match msg.role {
        MessageRole::User => ChatRole::User,
        MessageRole::Assistant => ChatRole::Assistant,
        MessageRole::Tool => ChatRole::Tool,
        MessageRole::System => ChatRole::System,
    };
    let tool_calls = msg
        .tool_calls
        .as_deref()
        .and_then(|s| serde_json::from_str::<Vec<ToolCall>>(s).ok())
        .unwrap_or_default();
    ChatMessage {
        role,
        content: msg.content,
        tool_calls,
        tool_call_id: msg.tool_call_id,
    }
}

async fn resolve_provider(db: &Database, provider_id: &str) -> ProviderResult<Box<dyn Provider>> {
    let metadata = find_metadata(provider_id)
        .ok_or_else(|| ProviderError::NotImplemented(format!("unknown provider {provider_id}")))?;
    let api_key = keychain::load_api_key(provider_id)?;
    let config =
        sqlx::query_as::<_, ProviderConfig>("SELECT * FROM provider_configs WHERE provider_id = ?")
            .bind(provider_id)
            .fetch_optional(db.pool())
            .await?;

    build_provider(ProviderInputs {
        metadata,
        api_key,
        model: config.as_ref().and_then(|c| c.selected_model.clone()),
        custom_endpoint: config.and_then(|c| c.custom_endpoint),
    })
}
