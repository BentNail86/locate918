//! # LLM Integration Service
//!
//! This module handles communication with the Python LLM microservice
//! which powers natural language event discovery using Google Gemini.
//!
//! ## Owner
//! Ben (AI Engineer) - Python service implementation
//! Will (Backend Lead) - Rust client code
//!
//! ## Architecture
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        Chat Request Flow                        │
//! └─────────────────────────────────────────────────────────────────┘
//!
//!  User: "Any concerts this weekend?"
//!           │
//!           ▼
//!  ┌─────────────────┐
//!  │  Rust Backend   │  POST /api/chat
//!  │  (routes/chat)  │
//!  └────────┬────────┘
//!           │ HTTP
//!           ▼
//!  ┌─────────────────┐     ┌─────────────────┐
//!  │  Python LLM     │────▶│  Gemini API     │
//!  │  Service :8001  │◀────│  (external)     │
//!  └────────┬────────┘     └─────────────────┘
//!           │
//!           │ Returns: SearchParams or formatted response
//!           ▼
//!  ┌─────────────────┐
//!  │  Rust Backend   │
//!  │  (search DB)    │
//!  └────────┬────────┘
//!           │
//!           ▼
//!  ┌─────────────────┐
//!  │  Response to    │
//!  │  User           │
//!  └─────────────────┘
//! ```
//!
//! ## Environment Variables
//! ```text
//! LLM_SERVICE_URL=http://localhost:8001
//! ```
//!
//! ## Endpoints Called
//! | Python Endpoint | Purpose |
//! |-----------------|---------|
//! | POST /api/parse-intent | Convert natural language → search params |
//! | POST /api/chat | Generate conversational response |
//! | GET /health | Health check |

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use crate::models::Event;

// =============================================================================
// CONFIGURATION
// =============================================================================

/// Get the LLM service URL from environment or default to localhost
fn get_llm_service_url() -> String {
    env::var("LLM_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8001".to_string())
}

// =============================================================================
// DATA STRUCTURES
// =============================================================================

/// Parameters extracted from user's natural language query.
///
/// The Python LLM service returns this after parsing a message like
/// "Any jazz concerts downtown this Friday?"
///
/// # Example
/// ```json
/// {
///   "query": "jazz",
///   "category": "music",
///   "location": "downtown",
///   "date_from": "2026-01-24"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchParams {
    /// Text to search for in event titles/descriptions
    pub query: Option<String>,

    /// Category filter (e.g., "music", "sports", "food")
    pub category: Option<String>,

    /// Start of date range (YYYY-MM-DD)
    pub date_from: Option<String>,

    /// End of date range (YYYY-MM-DD)
    pub date_to: Option<String>,

    /// Location filter (e.g., "downtown", "midtown")
    pub location: Option<String>,
}

// -----------------------------------------------------------------------------
// Request/Response types for Python service communication
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ParseIntentRequest {
    message: String,
}

#[derive(Debug, Deserialize)]
struct ParseIntentResponse {
    params: SearchParams,
    #[allow(dead_code)]
    confidence: f32,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    message: String,
    user_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    events: Option<Vec<Event>>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    reply: String,
    #[allow(dead_code)]
    events: Vec<Event>,
    search_params: Option<SearchParams>,
}

// =============================================================================
// ERROR TYPE
// =============================================================================

/// Errors that can occur when communicating with the LLM service.
#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("LLM service returned error: {0}")]
    ServiceError(String),

    #[error("LLM service unavailable")]
    ServiceUnavailable,
}

// =============================================================================
// LLM CLIENT
// =============================================================================

/// Client for communicating with the Python LLM service.
///
/// # Example
/// ```rust
/// let client = LlmClient::new();
///
/// // Check if service is running
/// if client.health_check().await? {
///     // Parse user's natural language query
///     let params = client.parse_intent("Any jazz concerts this weekend?").await?;
///     println!("Category: {:?}", params.category);
/// }
/// ```
pub struct LlmClient {
    client: Client,
    base_url: String,
}

impl LlmClient {
    /// Create a new LLM client.
    ///
    /// Reads `LLM_SERVICE_URL` from environment, defaults to `http://localhost:8001`.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: get_llm_service_url(),
        }
    }

    /// Check if the LLM service is healthy and ready to accept requests.
    ///
    /// # Returns
    /// - `Ok(true)` if service is healthy
    /// - `Ok(false)` if service responded but isn't ready
    /// - `Err(LlmError)` if service is unreachable
    pub async fn health_check(&self) -> Result<bool, LlmError> {
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;
        Ok(response.status().is_success())
    }

    /// Parse a natural language query into structured search parameters.
    ///
    /// # Arguments
    /// * `message` - User's natural language query
    ///
    /// # Returns
    /// * `Ok(SearchParams)` - Extracted search parameters
    /// * `Err(LlmError)` - If the service call fails
    ///
    /// # Example
    /// ```rust
    /// let params = client.parse_intent("Any jazz concerts downtown this Friday?").await?;
    /// // params.category = Some("music")
    /// // params.query = Some("jazz")
    /// // params.location = Some("downtown")
    /// // params.date_from = Some("2026-01-24")
    /// ```
    pub async fn parse_intent(&self, message: &str) -> Result<SearchParams, LlmError> {
        let url = format!("{}/api/parse-intent", self.base_url);

        let request = ParseIntentRequest {
            message: message.to_string(),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ServiceError(error_text));
        }

        let parsed: ParseIntentResponse = response.json().await?;
        Ok(parsed.params)
    }

    /// Generate a conversational response about events.
    ///
    /// Called AFTER searching the database with params from `parse_intent`.
    ///
    /// # Arguments
    /// * `message` - Original user query
    /// * `events` - Events found in the database
    /// * `user_id` - Optional user ID for personalization
    ///
    /// # Returns
    /// * `Ok(String)` - Conversational response from the LLM
    /// * `Err(LlmError)` - If the service call fails
    pub async fn generate_response(
        &self,
        message: &str,
        events: Vec<Event>,
        user_id: Option<uuid::Uuid>,
    ) -> Result<String, LlmError> {
        let url = format!("{}/api/chat", self.base_url);

        let request = ChatRequest {
            message: message.to_string(),
            user_id,
            events: Some(events),
        };

        let response = self.client.post(&url).json(&request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LlmError::ServiceError(error_text));
        }

        let chat_response: ChatResponse = response.json().await?;
        Ok(chat_response.reply)
    }
}

impl Default for LlmClient {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// CONVENIENCE FUNCTIONS
// =============================================================================

/// Parses a natural language query into structured search parameters.
///
/// Convenience function that creates a client and calls `parse_intent`.
///
/// # Example
/// ```rust
/// let params = parse_user_intent("What's happening this weekend?").await?;
/// ```
pub async fn parse_user_intent(message: &str) -> Result<SearchParams, LlmError> {
    let client = LlmClient::new();
    client.parse_intent(message).await
}

/// Processes a chat message and returns a conversational response with events.
///
/// This is the main entry point called by `routes/chat.rs`.
///
/// # Flow
/// 1. Parse user intent to get search params
/// 2. Search database with those params
/// 3. Pass events to LLM for formatting
/// 4. Return conversational response + events
///
/// # Arguments
/// * `user_id` - User's UUID (for personalization)
/// * `message` - User's chat message
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok((String, Vec<Event>))` - (LLM response, matching events)
/// * `Err(...)` - If any step fails
pub async fn process_chat_message(
    user_id: uuid::Uuid,
    message: &str,
    pool: &sqlx::PgPool,
) -> Result<(String, Vec<Event>), Box<dyn std::error::Error + Send + Sync>> {
    let client = LlmClient::new();

    // Step 1: Parse intent to get search parameters
    let params = client.parse_intent(message).await?;

    // Step 2: Search database with extracted parameters
    let events = search_events_with_params(&params, pool).await?;

    // Step 3: Generate conversational response
    let reply = client
        .generate_response(message, events.clone(), Some(user_id))
        .await?;

    Ok((reply, events))
}

/// Search events using the extracted parameters.
///
/// Mirrors the logic in `routes/events.rs` search_events handler.
async fn search_events_with_params(
    params: &SearchParams,
    pool: &sqlx::PgPool,
) -> Result<Vec<Event>, sqlx::Error> {
    match (&params.query, &params.category) {
        // Both query text and category
        (Some(q), Some(cat)) => {
            let search = format!("%{}%", q);
            sqlx::query_as::<_, Event>(
                r#"
                SELECT id, title, description, location, venue, source_url,
                       start_time, end_time, category, created_at
                FROM events
                WHERE (title ILIKE $1 OR description ILIKE $1) AND category = $2
                ORDER BY start_time ASC
                "#,
            )
                .bind(&search)
                .bind(cat)
                .fetch_all(pool)
                .await
        }

        // Only query text
        (Some(q), None) => {
            let search = format!("%{}%", q);
            sqlx::query_as::<_, Event>(
                r#"
                SELECT id, title, description, location, venue, source_url,
                       start_time, end_time, category, created_at
                FROM events
                WHERE title ILIKE $1 OR description ILIKE $1
                ORDER BY start_time ASC
                "#,
            )
                .bind(&search)
                .fetch_all(pool)
                .await
        }

        // Only category
        (None, Some(cat)) => {
            sqlx::query_as::<_, Event>(
                r#"
                SELECT id, title, description, location, venue, source_url,
                       start_time, end_time, category, created_at
                FROM events
                WHERE category = $1
                ORDER BY start_time ASC
                "#,
            )
                .bind(cat)
                .fetch_all(pool)
                .await
        }

        // No filters - return recent events
        (None, None) => {
            sqlx::query_as::<_, Event>(
                r#"
                SELECT id, title, description, location, venue, source_url,
                       start_time, end_time, category, created_at
                FROM events
                ORDER BY start_time ASC
                LIMIT 20
                "#,
            )
                .fetch_all(pool)
                .await
        }
    }
}