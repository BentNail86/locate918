"""
Locate918 LLM Service - Gemini Integration
==========================================
Owner: Ben (AI Engineer)

Core integration with Google's Gemini API.

Setup:
1. Get API key at https://makersuite.google.com/app/apikey
2. pip install google-generativeai
3. Add GEMINI_API_KEY to .env

Functions to implement:
- parse_user_intent(message) → SearchParams
- generate_chat_response(message, events, user_profile) → str
- normalize_events(raw_events) → List[NormalizedEvent]

See backend/src/services/llm.rs for the Rust client that calls these.
"""

# TODO: Ben to implement