from pydantic import BaseModel
from typing import List, Optional, Dict, Any

# Search Schemas

class SearchRequest(BaseModel):
    query: str

class SearchResponse(BaseModel):
    parsed_params: Dict[str, Any]
    # For now just returns the parsed intent.

# Chat Schemas

class ChatRequest(BaseModel):
    message: str
    user_id: str
    conversation_history: Optional[List[Dict[str, Any]]] = []
    # Example history item: {"role": "user", "parts": ["Hello"]}

class ChatResponse(BaseModel):
    text: Optional[str]
    tool_call: Optional[Dict[str, Any]]

# Normalization Schemas

class NormalizeRequest(BaseModel):
    raw_html: str
    source_url: str

class NormalizeResponse(BaseModel):
    events: List[Dict[str, Any]]