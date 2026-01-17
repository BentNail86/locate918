//! # Users Routes
//!
//! This module handles all user-related API endpoints including:
//! - User account management
//! - Preference tracking (what categories users like/dislike)
//! - Interaction tracking (what events users view, save, attend)
//! - User profiles (aggregated data for LLM personalization)
//!
//! ## Purpose
//! The user system powers personalized recommendations. When Ben's LLM
//! integration asks "What should this user do this weekend?", it uses
//! the UserProfile endpoint to understand:
//! - What categories they prefer (music +5, sports -2)
//! - What events they've interacted with recently
//! - Their location preferences
//!
//! ## Endpoints
//! - `POST /api/users`                    - Create new user account
//! - `GET  /api/users/:id`                - Get basic user info
//! - `GET  /api/users/:id/profile`        - Get full profile (for LLM)
//! - `GET  /api/users/:id/preferences`    - List category preferences
//! - `POST /api/users/:id/preferences`    - Add/update a preference
//! - `GET  /api/users/:id/interactions`   - List event interactions
//! - `POST /api/users/:id/interactions`   - Record new interaction
//!
//! ## Owner
//! Will (Coordinator/Backend Lead)

// =============================================================================
// IMPORTS
// =============================================================================

use axum::{
    extract::{Path, State},   // Extract data from requests
    http::StatusCode,          // HTTP status codes
    routing::{get, post},      // Route method helpers
    Json, Router,              // JSON handling and routing
};
use sqlx::PgPool;              // PostgreSQL connection pool
use uuid::Uuid;                // UUID type for IDs

// Import all our user-related models
use crate::models::{
    CreateUser, CreateUserInteraction, CreateUserPreference,
    User, UserPreference, UserInteraction, UserProfile, UserInteractionWithEvent,
};

// =============================================================================
// ROUTE DEFINITIONS
// =============================================================================

/// Creates the router for all user endpoints.
///
/// # Route Structure
/// ```text
/// /users
/// ├── POST /                    -> create_user()       - Create account
/// ├── GET  /:id                 -> get_user()          - Get user info
/// ├── GET  /:id/profile         -> get_user_profile()  - Full profile for LLM
/// ├── GET  /:id/preferences     -> get_preferences()   - List preferences
/// ├── POST /:id/preferences     -> add_preference()    - Add/update preference
/// ├── GET  /:id/interactions    -> get_interactions()  - List interactions
/// └── POST /:id/interactions    -> add_interaction()   - Record interaction
/// ```
pub fn routes() -> Router<PgPool> {
    Router::new()
        // Create a new user
        .route("/", post(create_user))
        // Get basic user info
        .route("/:id", get(get_user))
        // Get full user profile (used by LLM for personalization)
        .route("/:id/profile", get(get_user_profile))
        // Manage user preferences (category likes/dislikes)
        .route("/:id/preferences", get(get_preferences).post(add_preference))
        // Track user interactions with events
        .route("/:id/interactions", get(get_interactions).post(add_interaction))
}

// =============================================================================
// HANDLER: CREATE USER
// =============================================================================

/// Creates a new user account.
///
/// # Endpoint
/// `POST /api/users`
///
/// # Request Body
/// ```json
/// {
///   "email": "user@example.com",
///   "name": "John Doe",
///   "location_preference": "downtown"
/// }
/// ```
///
/// # Returns
/// - `201 Created` with the new user object
/// - `500 Internal Server Error` if creation fails (e.g., duplicate email)
///
/// # Notes
/// - Email must be unique (enforced by database constraint)
/// - Name and location_preference are optional
/// - In production, add proper authentication (OAuth, password hashing, etc.)
async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    // Generate unique ID and timestamp
    let id = Uuid::new_v4();
    let created_at = chrono::Utc::now();

    // Insert new user into database
    sqlx::query(
        r#"
        INSERT INTO users (id, email, name, location_preference, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
        .bind(&id)
        .bind(&payload.email)              // Required - must be unique
        .bind(&payload.name)               // Optional - user's display name
        .bind(&payload.location_preference) // Optional - preferred area (e.g., "downtown")
        .bind(&created_at)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Build response object
    let user = User {
        id,
        email: payload.email,
        name: payload.name,
        location_preference: payload.location_preference,
        created_at,
    };

    Ok((StatusCode::CREATED, Json(user)))
}

// =============================================================================
// HANDLER: GET USER
// =============================================================================

/// Retrieves basic user information by ID.
///
/// # Endpoint
/// `GET /api/users/:id`
///
/// # Returns
/// - `200 OK` with user object
/// - `404 Not Found` if user doesn't exist
/// - `500 Internal Server Error` if database query fails
///
/// # Example Response
/// ```json
/// {
///   "id": "94c99eb0-21f3-4f7e-afee-f533b964a2d4",
///   "email": "will@test.com",
///   "name": "Will",
///   "location_preference": "downtown",
///   "created_at": "2026-01-17T19:34:01Z"
/// }
/// ```
async fn get_user(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<User>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, name, location_preference, created_at FROM users WHERE id = $1"
    )
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match user {
        Some(u) => Ok(Json(u)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// =============================================================================
// HANDLER: GET USER PROFILE (FOR LLM)
// =============================================================================

/// Retrieves a complete user profile for LLM personalization.
///
/// # Endpoint
/// `GET /api/users/:id/profile`
///
/// # Purpose
/// This is the KEY endpoint for AI personalization. When Ben's chat endpoint
/// receives a query like "What should I do this weekend?", it calls this
/// endpoint to understand the user's preferences and history.
///
/// # Returns
/// - `200 OK` with complete profile including:
///   - Basic user info
///   - All category preferences (with weights)
///   - Recent 20 event interactions (with event details)
/// - `404 Not Found` if user doesn't exist
/// - `500 Internal Server Error` if database query fails
///
/// # Example Response
/// ```json
/// {
///   "user": {
///     "id": "94c99eb0-...",
///     "email": "will@test.com",
///     "name": "Will",
///     "location_preference": "downtown"
///   },
///   "preferences": [
///     { "category": "music", "weight": 5 },
///     { "category": "sports", "weight": -2 }
///   ],
///   "recent_interactions": [
///     {
///       "interaction_type": "view",
///       "event_title": "Jazz Night",
///       "event_category": "music",
///       "created_at": "2026-01-17T20:00:00Z"
///     }
///   ]
/// }
/// ```
///
/// # How the LLM Uses This
/// ```text
/// User asks: "What's happening this weekend?"
///
/// LLM sees profile:
///   - Likes music (weight: +5)
///   - Dislikes sports (weight: -2)
///   - Prefers downtown
///   - Recently viewed jazz events
///
/// LLM thinks: "I should prioritize music events downtown,
///              avoid sports, and maybe suggest more jazz."
/// ```
async fn get_user_profile(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserProfile>, StatusCode> {

    // Step 1: Get basic user info
    let user = sqlx::query_as::<_, User>(
        "SELECT id, email, name, location_preference, created_at FROM users WHERE id = $1"
    )
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;  // Return 404 if user not found

    // Step 2: Get all category preferences
    // Positive weights = likes, negative weights = dislikes
    let preferences = sqlx::query_as::<_, UserPreference>(
        "SELECT id, user_id, category, weight, created_at FROM user_preferences WHERE user_id = $1"
    )
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Step 3: Get recent interactions WITH event details
    // This JOIN gives us event info alongside the interaction
    // Limited to 20 most recent to keep response size reasonable
    let recent_interactions = sqlx::query_as::<_, UserInteractionWithEvent>(
        r#"
        SELECT ui.interaction_type, e.title as event_title, e.category as event_category, ui.created_at
        FROM user_interactions ui
        JOIN events e ON ui.event_id = e.id
        WHERE ui.user_id = $1
        ORDER BY ui.created_at DESC
        LIMIT 20
        "#
    )
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Combine everything into the profile response
    Ok(Json(UserProfile {
        user,
        preferences,
        recent_interactions,
    }))
}

// =============================================================================
// HANDLER: GET PREFERENCES
// =============================================================================

/// Lists all category preferences for a user.
///
/// # Endpoint
/// `GET /api/users/:id/preferences`
///
/// # Returns
/// - `200 OK` with array of preferences
///
/// # Example Response
/// ```json
/// [
///   { "id": "...", "user_id": "...", "category": "music", "weight": 5 },
///   { "id": "...", "user_id": "...", "category": "sports", "weight": -2 }
/// ]
/// ```
///
/// # Weight Scale (suggested)
/// - +5 = Love it
/// - +3 = Like it
/// - +1 = Slightly interested
/// -  0 = Neutral (or just don't create a preference)
/// - -1 = Slightly disinterested
/// - -3 = Dislike
/// - -5 = Hate it / never show me this
async fn get_preferences(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<UserPreference>>, StatusCode> {
    let preferences = sqlx::query_as::<_, UserPreference>(
        "SELECT id, user_id, category, weight, created_at FROM user_preferences WHERE user_id = $1"
    )
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(preferences))
}

// =============================================================================
// HANDLER: ADD/UPDATE PREFERENCE
// =============================================================================

/// Adds or updates a category preference for a user.
///
/// # Endpoint
/// `POST /api/users/:id/preferences`
///
/// # Request Body
/// ```json
/// {
///   "category": "music",
///   "weight": 5
/// }
/// ```
///
/// # Behavior
/// - If preference for this category doesn't exist: creates it
/// - If preference for this category exists: updates the weight
/// - Uses PostgreSQL's `ON CONFLICT ... DO UPDATE` for atomic upsert
///
/// # Returns
/// - `201 Created` with the preference object
///
/// # Example Usage
/// ```text
/// User clicks "I like music" button in the app
///   -> Frontend calls: POST /api/users/:id/preferences
///      Body: { "category": "music", "weight": 5 }
///
/// User clicks "Not interested in sports"
///   -> Frontend calls: POST /api/users/:id/preferences
///      Body: { "category": "sports", "weight": -3 }
/// ```
async fn add_preference(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<CreateUserPreference>,
) -> Result<(StatusCode, Json<UserPreference>), StatusCode> {
    let id = Uuid::new_v4();
    let created_at = chrono::Utc::now();

    // UPSERT: Insert new preference, or update weight if category exists
    // ON CONFLICT (user_id, category) - triggered when this combo already exists
    // DO UPDATE SET weight = $4 - update the weight to the new value
    sqlx::query(
        r#"
        INSERT INTO user_preferences (id, user_id, category, weight, created_at)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (user_id, category) DO UPDATE SET weight = $4
        "#,
    )
        .bind(&id)
        .bind(&user_id)
        .bind(&payload.category)
        .bind(&payload.weight)
        .bind(&created_at)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let preference = UserPreference {
        id,
        user_id,
        category: payload.category,
        weight: payload.weight,
        created_at,
    };

    Ok((StatusCode::CREATED, Json(preference)))
}

// =============================================================================
// HANDLER: GET INTERACTIONS
// =============================================================================

/// Lists all event interactions for a user.
///
/// # Endpoint
/// `GET /api/users/:id/interactions`
///
/// # Returns
/// - `200 OK` with array of interactions (newest first)
///
/// # Interaction Types
/// - `"view"` - User viewed the event details
/// - `"save"` - User saved/bookmarked the event
/// - `"attend"` - User marked as attending
/// - `"dismiss"` - User dismissed/hid the event
///
/// # Example Response
/// ```json
/// [
///   {
///     "id": "...",
///     "user_id": "...",
///     "event_id": "...",
///     "interaction_type": "view",
///     "created_at": "2026-01-17T20:00:00Z"
///   }
/// ]
/// ```
async fn get_interactions(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<UserInteraction>>, StatusCode> {
    let interactions = sqlx::query_as::<_, UserInteraction>(
        "SELECT id, user_id, event_id, interaction_type, created_at FROM user_interactions WHERE user_id = $1 ORDER BY created_at DESC"
    )
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(interactions))
}

// =============================================================================
// HANDLER: ADD INTERACTION
// =============================================================================

/// Records a new user interaction with an event.
///
/// # Endpoint
/// `POST /api/users/:id/interactions`
///
/// # Request Body
/// ```json
/// {
///   "event_id": "550e8400-e29b-41d4-a716-446655440000",
///   "interaction_type": "view"
/// }
/// ```
///
/// # Returns
/// - `201 Created` with the interaction object
///
/// # Purpose
/// Tracking interactions enables implicit preference learning:
/// - User views lots of music events? They probably like music.
/// - User dismisses sports events? They probably don't like sports.
/// - User attends food festivals? Recommend more food events.
///
/// # Example Usage
/// ```text
/// User opens event detail page
///   -> Frontend calls: POST /api/users/:id/interactions
///      Body: { "event_id": "...", "interaction_type": "view" }
///
/// User clicks "Save Event" button
///   -> Frontend calls: POST /api/users/:id/interactions
///      Body: { "event_id": "...", "interaction_type": "save" }
///
/// User clicks "Not Interested"
///   -> Frontend calls: POST /api/users/:id/interactions
///      Body: { "event_id": "...", "interaction_type": "dismiss" }
/// ```
///
/// # Future Enhancement
/// Malachi's frontend will call this automatically when users interact
/// with events in the UI. Jordi can build analytics on top of this data.
async fn add_interaction(
    State(pool): State<PgPool>,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<CreateUserInteraction>,
) -> Result<(StatusCode, Json<UserInteraction>), StatusCode> {
    let id = Uuid::new_v4();
    let created_at = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO user_interactions (id, user_id, event_id, interaction_type, created_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
        .bind(&id)
        .bind(&user_id)
        .bind(&payload.event_id)
        .bind(&payload.interaction_type)
        .bind(&created_at)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let interaction = UserInteraction {
        id,
        user_id,
        event_id: payload.event_id,
        interaction_type: payload.interaction_type,
        created_at,
    };

    Ok((StatusCode::CREATED, Json(interaction)))
}