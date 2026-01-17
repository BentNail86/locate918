//! # LLM Integration Service
//!
//! This module handles all communication with the Large Language Model (Gemini)
//! to power natural language event discovery.
//!
//! ## Owner
//! Ben (AI Engineer)
//!
//! ## Overview
//! Instead of users searching with filters like "category=music&date=2026-01-24",
//! they can ask naturally: "What's happening downtown this weekend?"
//!
//! This service:
//! 1. Takes the user's natural language query
//! 2. Sends it to Gemini with context (user profile, available tools)
//! 3. Gemini decides what searches to perform
//! 4. We execute those searches against our database
//! 5. Gemini formats the results conversationally
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
//!  │  /api/chat      │  (routes/chat.rs)
//!  │  endpoint       │
//!  └────────┬────────┘
//!           │
//!           ▼
//!  ┌─────────────────┐     ┌─────────────────┐
//!  │  LLM Service    │────▶│  Gemini API     │
//!  │  (this file)    │◀────│  (external)     │
//!  └────────┬────────┘     └─────────────────┘
//!           │
//!           │ Gemini says: "Call search_events(category='music')"
//!           ▼
//!  ┌─────────────────┐
//!  │  Internal       │
//!  │  Event Search   │
//!  └────────┬────────┘
//!           │
//!           │ Returns: [Event1, Event2, Event3]
//!           ▼
//!  ┌─────────────────┐     ┌─────────────────┐
//!  │  LLM Service    │────▶│  Gemini API     │
//!  │  (format result)│◀────│  (external)     │
//!  └────────┬────────┘     └─────────────────┘
//!           │
//!           │ Gemini says: "I found 3 concerts this weekend!"
//!           ▼
//!  ┌─────────────────┐
//!  │  Response to    │
//!  │  User           │
//!  └─────────────────┘
//! ```
//!
//! ## Tool Use / Function Calling
//! Modern LLMs support "tool use" - we tell the LLM what functions it can call,
//! and it decides when to use them. For Locate918, the tools are:
//!
//! | Tool | Description | Parameters |
//! |------|-------------|------------|
//! | search_events | Search for events | query, category, date_from, date_to, location |
//! | get_event | Get event details | event_id |
//! | list_categories | List available categories | (none) |
//!
//! ## Personalization
//! Before calling the LLM, we fetch the user's profile (preferences, history).
//! This context helps the LLM make personalized recommendations:
//!
//! ```text
//! System Prompt:
//!   "User Profile:
//!    - Likes: music (+5), food (+3)
//!    - Dislikes: sports (-2)
//!    - Location preference: downtown
//!    - Recently viewed: Jazz Night, Food Truck Festival"
//! ```
//!
//! ## Getting Started (Ben's Implementation Guide)
//!
//! ### Step 1: Get Gemini API Key
//! 1. Go to https://makersuite.google.com/app/apikey
//! 2. Create a new API key
//! 3. Add to backend/.env: GEMINI_API_KEY=your_key_here
//!
//! ### Step 2: Implement query_llm()
//! Basic HTTP call to Gemini API using reqwest.
//!
//! ### Step 3: Implement parse_user_intent()
//! Use Gemini to extract structured search parameters from natural language.
//!
//! ### Step 4: Implement the tool use loop
//! Handle Gemini's requests to call our search functions.
//!
//! ### Step 5: Connect to routes/chat.rs
//! Wire up the /api/chat endpoint to use this service.
//!
//! ## Environment Variables
//! ```text
//! GEMINI_API_KEY=your_api_key_here
//! ```
//!
//! ## Dependencies
//! Already in Cargo.toml:
//! - `reqwest` - HTTP client for API calls
//! - `serde_json` - JSON serialization for API payloads

// =============================================================================
// IMPORTS (uncomment when implementing)
// =============================================================================

// use reqwest::Client;
// use serde::{Deserialize, Serialize};
// use std::env;

// =============================================================================
// CONFIGURATION
// =============================================================================

/// Gemini API endpoint for chat completions
/// Docs: https://ai.google.dev/api/rest/v1beta/models/generateContent
#[allow(dead_code)]
const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent";

// =============================================================================
// DATA STRUCTURES (uncomment and expand when implementing)
// =============================================================================

// /// Parameters extracted from user's natural language query
// #[derive(Debug, Serialize, Deserialize)]
// pub struct SearchParams {
//     /// Text to search for in event titles/descriptions
//     pub query: Option<String>,
//
//     /// Category filter (e.g., "music", "sports")
//     pub category: Option<String>,
//
//     /// Start of date range
//     pub date_from: Option<String>,
//
//     /// End of date range
//     pub date_to: Option<String>,
//
//     /// Location filter (e.g., "downtown")
//     pub location: Option<String>,
// }

// /// Request body for Gemini API
// #[derive(Debug, Serialize)]
// struct GeminiRequest {
//     contents: Vec<GeminiContent>,
// }

// #[derive(Debug, Serialize)]
// struct GeminiContent {
//     parts: Vec<GeminiPart>,
// }

// #[derive(Debug, Serialize)]
// struct GeminiPart {
//     text: String,
// }

// /// Response from Gemini API
// #[derive(Debug, Deserialize)]
// struct GeminiResponse {
//     candidates: Vec<GeminiCandidate>,
// }

// #[derive(Debug, Deserialize)]
// struct GeminiCandidate {
//     content: GeminiContentResponse,
// }

// #[derive(Debug, Deserialize)]
// struct GeminiContentResponse {
//     parts: Vec<GeminiPartResponse>,
// }

// #[derive(Debug, Deserialize)]
// struct GeminiPartResponse {
//     text: String,
// }

// =============================================================================
// SYSTEM PROMPT
// =============================================================================

/// System prompt that defines how the LLM should behave.
/// This tells Gemini about Locate918 and what tools it can use.
#[allow(dead_code)]
const SYSTEM_PROMPT: &str = r#"
You are a helpful assistant for Locate918, an event discovery app for the Tulsa (918) area.

Your job is to help users find local events based on their interests and queries.

## Available Tools

You can search for events using these parameters:
- query: Text to search in event titles and descriptions
- category: Filter by category (music, sports, food, arts, community, nightlife)
- date_from / date_to: Filter by date range
- location: Filter by area (downtown, midtown, etc.)

## User Profile

When provided, use the user's profile to personalize recommendations:
- Prioritize categories they like (positive weight)
- Avoid categories they dislike (negative weight)
- Consider their location preference
- Reference their recent activity when relevant

## Response Guidelines

1. Be conversational and friendly
2. Always mention the event name, date/time, and venue
3. Include a brief description of why they might like it
4. If no events match, suggest broadening the search
5. Offer to help find more specific events

## Example Interaction

User: "What's happening this weekend?"

Response: "I found some great events this weekend! 🎵

**Friday Night:**
- Jazz at the Blue Note (8 PM) - Great live jazz downtown, perfect for a chill evening

**Saturday:**
- Tulsa Food Truck Festival (11 AM - 4 PM) - Over 20 food trucks at Gathering Place
- OSU vs Kansas Basketball (7 PM) - Big game at Gallagher-Iba Arena

Want me to find more events in a specific category?"
"#;

// =============================================================================
// MAIN FUNCTIONS (Ben to implement)
// =============================================================================

/// Sends a prompt to the Gemini API and returns the response.
///
/// # Arguments
/// * `prompt` - The user's message or query
///
/// # Returns
/// * `Ok(String)` - The LLM's response text
/// * `Err(...)` - If the API call fails
///
/// # Example
/// ```rust
/// let response = query_llm("What concerts are happening this weekend?").await?;
/// println!("{}", response);
/// ```
///
/// # Implementation Notes
/// 1. Get API key from environment: `env::var("GEMINI_API_KEY")`
/// 2. Build request with system prompt + user message
/// 3. Send POST request to GEMINI_API_URL
/// 4. Parse response and extract text
///
/// # API Documentation
/// https://ai.google.dev/api/rest/v1beta/models/generateContent
#[allow(dead_code)]
pub async fn query_llm(_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    // TODO: Ben to implement
    //
    // Example implementation:
    //
    // let api_key = env::var("GEMINI_API_KEY")?;
    // let client = Client::new();
    //
    // let request = GeminiRequest {
    //     contents: vec![
    //         GeminiContent {
    //             parts: vec![
    //                 GeminiPart { text: SYSTEM_PROMPT.to_string() },
    //                 GeminiPart { text: prompt.to_string() },
    //             ],
    //         },
    //     ],
    // };
    //
    // let response = client
    //     .post(format!("{}?key={}", GEMINI_API_URL, api_key))
    //     .json(&request)
    //     .send()
    //     .await?
    //     .json::<GeminiResponse>()
    //     .await?;
    //
    // let text = response.candidates
    //     .first()
    //     .and_then(|c| c.content.parts.first())
    //     .map(|p| p.text.clone())
    //     .unwrap_or_default();
    //
    // Ok(text)

    todo!("Ben to implement - see comments above for guidance")
}

/// Parses a natural language query into structured search parameters.
///
/// # Arguments
/// * `message` - The user's natural language query
///
/// # Returns
/// * `Ok(SearchParams)` - Extracted search parameters
/// * `Err(...)` - If parsing fails
///
/// # Example
/// ```rust
/// let params = parse_user_intent("Any jazz concerts downtown this Friday?").await?;
/// // params.category = Some("music")
/// // params.query = Some("jazz")
/// // params.location = Some("downtown")
/// // params.date_from = Some("2026-01-24")
/// ```
///
/// # Implementation Notes
/// Use Gemini to extract structured data from the query.
/// Prompt should ask for JSON output with specific fields.
///
/// Example prompt:
/// ```text
/// Extract search parameters from this query: "{message}"
///
/// Return JSON with these optional fields:
/// - query: keywords to search for
/// - category: music, sports, food, arts, community, nightlife
/// - date_from: start date (YYYY-MM-DD)
/// - date_to: end date (YYYY-MM-DD)
/// - location: area like "downtown", "midtown"
///
/// Only include fields that are clearly specified or implied.
/// ```
#[allow(dead_code)]
pub async fn parse_user_intent(_message: &str) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Ben to implement
    //
    // This function should:
    // 1. Send the message to Gemini with a prompt asking for JSON extraction
    // 2. Parse the JSON response into SearchParams
    // 3. Return the structured parameters
    //
    // Consider edge cases:
    // - "this weekend" -> calculate actual dates
    // - "tonight" -> today's date, evening time
    // - "something fun" -> might not have specific category

    todo!("Ben to implement - see comments above for guidance")
}

/// Processes a chat message with full tool use support.
///
/// This is the main entry point for the chat endpoint.
/// It handles the full conversation loop:
/// 1. Fetch user profile for personalization
/// 2. Send query to LLM with available tools
/// 3. Execute any tool calls (searches) the LLM requests
/// 4. Send results back to LLM for formatting
/// 5. Return final conversational response
///
/// # Arguments
/// * `user_id` - The user's UUID (for fetching profile)
/// * `message` - The user's chat message
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Ok((String, Vec<Event>))` - (LLM response text, matching events)
/// * `Err(...)` - If any step fails
///
/// # Example
/// ```rust
/// let (reply, events) = process_chat_message(user_id, "What's happening tonight?", &pool).await?;
/// // reply = "I found 3 events tonight! ..."
/// // events = [Event, Event, Event]
/// ```
#[allow(dead_code)]
pub async fn process_chat_message(
    _user_id: uuid::Uuid,
    _message: &str,
    _pool: &sqlx::PgPool,
) -> Result<(String, Vec<crate::models::Event>), Box<dyn std::error::Error>> {
    // TODO: Ben to implement
    //
    // High-level steps:
    //
    // 1. Fetch user profile
    //    let profile = sqlx::query_as::<_, UserProfile>(...)
    //
    // 2. Build context with profile info
    //    let context = format!("User preferences: {:?}", profile.preferences);
    //
    // 3. Send to LLM with tool definitions
    //    let llm_response = query_llm_with_tools(message, context, tools).await?;
    //
    // 4. If LLM wants to call a tool:
    //    let search_params = parse_tool_call(llm_response);
    //    let events = search_events(search_params, pool).await?;
    //
    // 5. Send events back to LLM for formatting
    //    let final_response = format_results_with_llm(events).await?;
    //
    // 6. Return response and events
    //    Ok((final_response, events))

    todo!("Ben to implement - see comments above for guidance")
}

// =============================================================================
// HELPER FUNCTIONS (Ben to implement as needed)
// =============================================================================

// /// Formats a list of events as context for the LLM
// fn format_events_for_llm(events: &[Event]) -> String {
//     events.iter()
//         .map(|e| format!("- {} at {} on {}", e.title, e.venue.as_deref().unwrap_or("TBD"), e.start_time))
//         .collect::<Vec<_>>()
//         .join("\n")
// }

// /// Formats user preferences as context for the LLM
// fn format_preferences_for_llm(preferences: &[UserPreference]) -> String {
//     preferences.iter()
//         .map(|p| {
//             let sentiment = if p.weight > 0 { "likes" } else { "dislikes" };
//             format!("- {} {} (strength: {})", sentiment, p.category, p.weight.abs())
//         })
//         .collect::<Vec<_>>()
//         .join("\n")
// }