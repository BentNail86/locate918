//! # Events Routes
//!
//! This module handles all event-related API endpoints.
//! Events are the core data type for Locate918 - they represent
//! local happenings that users want to discover.
//!
//! ## Endpoints
//! - `GET  /api/events`         - List all events (sorted by start time)
//! - `POST /api/events`         - Create a new event
//! - `GET  /api/events/:id`     - Get a single event by UUID
//! - `GET  /api/events/search`  - Search by keyword and/or category
//!
//! ## Owner
//! Will (Coordinator/Backend Lead)
//!
//! ## Future Enhancements (Skylar - Data Engineer)
//! - Events will be populated by web scrapers, not just manual creation
//! - Additional filters: date range, location radius, venue

// =============================================================================
// IMPORTS
// =============================================================================

use axum::{
    extract::{Path, Query, State},  // Extractors pull data from requests
    http::StatusCode,                // HTTP status codes (200, 404, 500, etc.)
    routing::get,                    // Route method helpers
    Json,                            // JSON request/response handling
    Router,                          // Router for defining routes
};
use serde::Deserialize;              // For deserializing JSON into structs
use sqlx::PgPool;                    // PostgreSQL connection pool
use uuid::Uuid;                      // UUID type for event IDs

use crate::models::{Event, CreateEvent};  // Our data models

// =============================================================================
// ROUTE DEFINITIONS
// =============================================================================

/// Creates the router for all event endpoints.
///
/// # Route Structure
/// ```text
/// /events
/// ├── GET  /           -> list_events()    - Get all events
/// ├── POST /           -> create_event()   - Create new event
/// ├── GET  /search     -> search_events()  - Search with filters
/// └── GET  /:id        -> get_event()      - Get event by ID
/// ```
///
/// # Note on Route Order
/// `/search` must come BEFORE `/:id` because Axum matches routes in order.
/// If `/:id` came first, "search" would be interpreted as an ID!
pub fn routes() -> Router<PgPool> {
    Router::new()
        // GET / and POST / share the same path but different methods
        .route("/", get(list_events).post(create_event))
        // Search endpoint - must be before /:id to avoid conflicts
        .route("/search", get(search_events))
        // Get single event by UUID
        .route("/:id", get(get_event))
}

// =============================================================================
// HANDLER: LIST ALL EVENTS
// =============================================================================

/// Returns all events in the database, sorted by start time (soonest first).
///
/// # Endpoint
/// `GET /api/events`
///
/// # Parameters
/// - `State(pool)`: Database connection pool (injected by Axum)
///
/// # Returns
/// - `200 OK` with JSON array of events
/// - `500 Internal Server Error` if database query fails
///
/// # Example Response
/// ```json
/// [
///   {
///     "id": "550e8400-e29b-41d4-a716-446655440000",
///     "title": "Jazz Night at The Blue Note",
///     "description": "Live jazz music featuring local artists",
///     "location": "Downtown Tulsa",
///     "venue": "The Blue Note",
///     "source_url": "https://example.com/event",
///     "start_time": "2026-01-25T20:00:00Z",
///     "end_time": "2026-01-25T23:00:00Z",
///     "category": "music",
///     "created_at": "2026-01-17T12:00:00Z"
///   }
/// ]
/// ```
async fn list_events(
    State(pool): State<PgPool>,  // Extract the database pool from app state
) -> Result<Json<Vec<Event>>, StatusCode> {

    // Execute SQL query to fetch all events
    // sqlx::query_as::<_, Event>() maps database rows to our Event struct
    // The underscore _ lets Rust infer the database type (Postgres)
    let events = sqlx::query_as::<_, Event>(
        "SELECT id, title, description, location, venue, source_url, start_time, end_time, category, created_at FROM events ORDER BY start_time ASC"
    )
        .fetch_all(&pool)  // Fetch all matching rows
        .await             // Await the async database operation
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;  // Convert DB errors to 500

    // Wrap the events vector in Json() for automatic serialization
    Ok(Json(events))
}

// =============================================================================
// HANDLER: GET SINGLE EVENT
// =============================================================================

/// Returns a single event by its UUID.
///
/// # Endpoint
/// `GET /api/events/:id`
///
/// # Parameters
/// - `State(pool)`: Database connection pool
/// - `Path(id)`: The event UUID from the URL path
///
/// # Returns
/// - `200 OK` with JSON event object if found
/// - `404 Not Found` if no event with that ID exists
/// - `500 Internal Server Error` if database query fails
///
/// # Example
/// `GET /api/events/550e8400-e29b-41d4-a716-446655440000`
async fn get_event(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,  // Extract UUID from URL path (e.g., /events/abc-123)
) -> Result<Json<Event>, StatusCode> {

    // Query for a single event by ID
    // $1 is a parameterized placeholder - prevents SQL injection
    let event = sqlx::query_as::<_, Event>(
        "SELECT id, title, description, location, venue, source_url, start_time, end_time, category, created_at FROM events WHERE id = $1"
    )
        .bind(id)              // Bind the UUID to the $1 placeholder
        .fetch_optional(&pool) // Returns Option<Event> - None if not found
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Handle the Option: return event or 404
    match event {
        Some(e) => Ok(Json(e)),           // Found - return 200 with event
        None => Err(StatusCode::NOT_FOUND), // Not found - return 404
    }
}

// =============================================================================
// HANDLER: CREATE EVENT
// =============================================================================

/// Creates a new event in the database.
///
/// # Endpoint
/// `POST /api/events`
///
/// # Parameters
/// - `State(pool)`: Database connection pool
/// - `Json(payload)`: Event data from request body (deserialized from JSON)
///
/// # Returns
/// - `201 Created` with the created event (including generated ID and timestamp)
/// - `500 Internal Server Error` if database insert fails
///
/// # Example Request Body
/// ```json
/// {
///   "title": "OSU Basketball Game",
///   "description": "Cowboys vs Kansas",
///   "location": "Stillwater, OK",
///   "venue": "Gallagher-Iba Arena",
///   "source_url": "https://okstate.com/events",
///   "start_time": "2026-02-01T19:00:00Z",
///   "category": "sports"
/// }
/// ```
///
/// # Note
/// This endpoint is primarily for testing. In production, most events
/// will be created by Skylar's scraper service, not manual API calls.
async fn create_event(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateEvent>,  // Deserialize JSON body into CreateEvent struct
) -> Result<(StatusCode, Json<Event>), StatusCode> {

    // Generate a new UUID for this event
    let id = Uuid::new_v4();

    // Record the current timestamp
    let created_at = chrono::Utc::now();

    // Insert the new event into the database
    // r#"..."# is a raw string literal - allows multiple lines and special chars
    sqlx::query(
        r#"
        INSERT INTO events (id, title, description, location, venue, source_url, start_time, end_time, category, created_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#,
    )
        .bind(&id)                    // $1 - event ID
        .bind(&payload.title)         // $2 - title (required)
        .bind(&payload.description)   // $3 - description (optional)
        .bind(&payload.location)      // $4 - location (optional)
        .bind(&payload.venue)         // $5 - venue name (optional)
        .bind(&payload.source_url)    // $6 - original source URL (required)
        .bind(&payload.start_time)    // $7 - when event starts (required)
        .bind(&payload.end_time)      // $8 - when event ends (optional)
        .bind(&payload.category)      // $9 - category like "music", "sports" (optional)
        .bind(&created_at)            // $10 - when we created this record
        .execute(&pool)               // Execute the INSERT (no rows returned)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Build the complete Event struct to return to the client
    // This includes the server-generated id and created_at
    let event = Event {
        id,
        title: payload.title,
        description: payload.description,
        location: payload.location,
        venue: payload.venue,
        source_url: payload.source_url,
        start_time: payload.start_time,
        end_time: payload.end_time,
        category: payload.category,
        created_at,
    };

    // Return 201 Created status with the new event
    Ok((StatusCode::CREATED, Json(event)))
}

// =============================================================================
// SEARCH QUERY PARAMETERS
// =============================================================================

/// Query parameters for the search endpoint.
///
/// Both fields are optional, allowing flexible search combinations:
/// - `/search` - Returns all events (same as list)
/// - `/search?q=jazz` - Text search in title and description
/// - `/search?category=music` - Filter by category
/// - `/search?q=jazz&category=music` - Combined search
#[derive(Deserialize)]
pub struct SearchQuery {
    /// Text to search for in event title and description
    /// Uses case-insensitive partial matching (SQL ILIKE with %)
    q: Option<String>,

    /// Category to filter by (exact match)
    /// Examples: "music", "sports", "food", "arts", "community"
    category: Option<String>,
}

// =============================================================================
// HANDLER: SEARCH EVENTS
// =============================================================================

/// Searches events by keyword and/or category.
///
/// # Endpoint
/// `GET /api/events/search`
///
/// # Query Parameters
/// - `q` (optional): Search text - matches against title and description
/// - `category` (optional): Filter by exact category match
///
/// # Returns
/// - `200 OK` with JSON array of matching events
/// - `500 Internal Server Error` if database query fails
///
/// # Search Behavior
/// - Text search (`q`) is case-insensitive and matches partial strings
/// - Category search is exact match
/// - When both provided, results must match BOTH criteria (AND logic)
/// - When neither provided, returns all events
///
/// # Examples
/// - `GET /api/events/search?q=basketball` - Events with "basketball" in title/description
/// - `GET /api/events/search?category=sports` - All sports events
/// - `GET /api/events/search?q=live&category=music` - Music events with "live" in text
///
/// # Future Enhancements
/// This is where Ben's LLM integration will shine - converting natural language
/// queries like "something chill this weekend" into appropriate search parameters.
async fn search_events(
    State(pool): State<PgPool>,
    Query(params): Query<SearchQuery>,  // Extract query params from URL
) -> Result<Json<Vec<Event>>, StatusCode> {

    // Match on the combination of parameters to build the right query
    // Each branch handles a different search scenario
    let events = match (&params.q, &params.category) {

        // CASE 1: Both search text AND category provided
        // Example: /search?q=jazz&category=music
        (Some(q), Some(cat)) => {
            // Wrap search term in % for ILIKE partial matching
            // "jazz" becomes "%jazz%" to match "Jazz Night", "Cool jazz", etc.
            let search = format!("%{}%", q);
            sqlx::query_as::<_, Event>(
                "SELECT id, title, description, location, venue, source_url, start_time, end_time, category, created_at FROM events WHERE (title ILIKE $1 OR description ILIKE $1) AND category = $2 ORDER BY start_time ASC"
            )
                .bind(&search)  // $1 - search pattern
                .bind(cat)      // $2 - exact category
                .fetch_all(&pool)
                .await
        }

        // CASE 2: Only search text provided (no category filter)
        // Example: /search?q=basketball
        (Some(q), None) => {
            let search = format!("%{}%", q);
            sqlx::query_as::<_, Event>(
                "SELECT id, title, description, location, venue, source_url, start_time, end_time, category, created_at FROM events WHERE title ILIKE $1 OR description ILIKE $1 ORDER BY start_time ASC"
            )
                .bind(&search)
                .fetch_all(&pool)
                .await
        }

        // CASE 3: Only category filter provided (no text search)
        // Example: /search?category=sports
        (None, Some(cat)) => {
            sqlx::query_as::<_, Event>(
                "SELECT id, title, description, location, venue, source_url, start_time, end_time, category, created_at FROM events WHERE category = $1 ORDER BY start_time ASC"
            )
                .bind(cat)
                .fetch_all(&pool)
                .await
        }

        // CASE 4: No parameters provided - return all events
        // Example: /search (equivalent to /events)
        (None, None) => {
            sqlx::query_as::<_, Event>(
                "SELECT id, title, description, location, venue, source_url, start_time, end_time, category, created_at FROM events ORDER BY start_time ASC"
            )
                .fetch_all(&pool)
                .await
        }
    }
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(events))
}