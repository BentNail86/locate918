"""
Locate918 LLM Service - Data Models
===================================
Owner: Ben (AI Engineer)

Pydantic models for API requests/responses.
These should mirror the Rust structs in backend/src/models/mod.rs.

Models to implement:
- SearchParams: query, category, date_from, date_to, location
- ParseIntentRequest/Response: for /api/parse-intent endpoint
- ChatRequest/Response: for /api/chat endpoint
- Event: mirrors Rust Event struct
- UserProfile: for personalization context
"""

# TODO: Ben to implement