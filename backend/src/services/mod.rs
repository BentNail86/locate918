//! # Services Module
//!
//! This module contains business logic and external service integrations.
//! Services are where the "smart" parts of the application live - they
//! coordinate between routes, database, and external APIs.
//!
//! ## Current Submodules
//! - `llm` - Large Language Model integration (Ben's domain)
//!
//! ## Architecture
//! ```text
//! ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//! │   Routes    │────▶│  Services   │────▶│  External   │
//! │  (HTTP)     │     │  (Logic)    │     │  APIs       │
//! └─────────────┘     └─────────────┘     └─────────────┘
//!                            │
//!                            ▼
//!                     ┌─────────────┐
//!                     │  Database   │
//!                     └─────────────┘
//! ```
//!
//! ## Why Services?
//! Separating business logic into services keeps routes thin and focused
//! on HTTP concerns. This makes the code:
//! - Easier to test (services can be tested without HTTP)
//! - Easier to reuse (multiple routes can use the same service)
//! - Easier to understand (clear separation of concerns)
//!
//! ## Future Services
//! As the app grows, consider adding:
//! - `notification` - Push notifications for saved events
//! - `analytics` - Track popular events, user trends
//! - `geocoding` - Convert addresses to coordinates for location search
//!
//! ## Owner
//! Will (Coordinator/Backend Lead) - module structure
//! Ben (AI Engineer) - LLM service implementation

// =============================================================================
// SUBMODULE DECLARATIONS
// =============================================================================

/// LLM (Large Language Model) integration service.
///
/// Handles communication with Gemini/Claude API for:
/// - Natural language event search
/// - Event summarization
/// - Personalized recommendations
///
/// Owner: Ben (AI Engineer)
pub mod llm;