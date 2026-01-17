"""
Locate918 LLM Service - Chat Routes
===================================
Owner: Ben (AI Engineer)

FastAPI endpoints called by the Rust backend.

Endpoints to implement:
- POST /api/parse-intent  → Convert natural language to SearchParams
- POST /api/chat          → Generate conversational response about events
- POST /api/normalize     → Clean up raw scraped event data (for Skylar)

Request flow:
1. User: "Any jazz concerts this weekend?"
2. Rust calls POST /api/parse-intent
3. Returns: {"params": {"category": "music", "query": "jazz"}}
4. Rust searches database
5. Rust calls POST /api/chat with message + events
6. Returns: {"reply": "I found 3 concerts! ..."}
"""

# TODO: Ben to implement