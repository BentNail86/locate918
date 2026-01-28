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

import os
import json
from typing import List, Dict, Any, Optional
import google.generativeai as genai
from google.generativeai.types import FunctionDeclaration, Tool
from dotenv import load_dotenv

load_dotenv()

API_KEY = os.getenv("GEMINI_API_KEY")
if not API_KEY:
    raise ValueError("GEMINI_API_KEY not found in environment variables")

genai.configure(api_key=API_KEY)

# Tool Definitions

search_events_tool_schema = {
    "name": "search_events",
    "description": "Search for events in the database based on criteria like category, date, price, etc.",
    "parameters": {
        "type": "object",
        "properties": {
            "q": {"type": "string", "description": "Keywords to search for in title or description"},
            "category": {"type": "string", "description": "Event category (concerts, sports, arts, food, family)"},
            "start_date": {"type": "string", "description": "ISO date string for start range"},
            "end_date": {"type": "string", "description": "ISO date string for end range"},
            "price_max": {"type": "number", "description": "Maximum price in dollars"},
            "location": {"type": "string", "description": "Area filter (e.g., Downtown, Broken Arrow, South Tulsa)"},
            "family_friendly": {"type": "boolean", "description": "Filter for family friendly events"},
            "outdoor": {"type": "boolean", "description": "Filter for outdoor events"}
        }
    }
}

# Correctly wrap in Tool object for the Gemini API
gemini_tools = [
    Tool(function_declarations=[FunctionDeclaration.from_dict(search_events_tool_schema)])
]

# Model Instantiation
# Instantiate the JSON-mode model for parsing and normalization.
# We use Gemini 2.0 Flash for its speed and improved instruction following.
json_model = genai.GenerativeModel(
    model_name='gemini-2.0-flash-exp',
    generation_config={"response_mime_type": "application/json"}
)


async def parse_user_intent(message: str) -> Dict[str, Any]:
    """
    Uses Gemini 2.0 Flash to extract search parameters from natural language.
    Returns a JSON object compatible with the backend search API.
    """
    prompt = f"""
    Extract search parameters from this query: "{message}"
    
    Output a valid JSON object with any of the following keys based on the query: 
    q, category, start_date, end_date, price_max, location, family_friendly, outdoor.
    Use null for missing fields.
    For dates, convert "this weekend" or "tomorrow" to approximate ISO dates based on current context.
    """
    
    response = await json_model.generate_content_async(prompt)
    try:
        return json.loads(response.text)
    except json.JSONDecodeError:
        return {"q": message}


async def generate_chat_response(message: str, history: List[Dict], user_profile: Dict) -> Dict[str, Any]:
    """
    Uses Gemini 2.0 Flash for conversation. Handles tool calling.
    Returns a dictionary containing the text response and any tool calls to be executed.
    """
    
    # Construct system prompt with user context
    system_instruction = f"""
    You are Tully, a friendly and knowledgeable event guide for Tulsa, OK (area code 918).
    User Profile: {json.dumps(user_profile)}
    
    If the user asks for events, use the `search_events` tool.
    If the user asks about weather or directions, answer generally or suggest checking a map.
    Be concise, enthusiastic, and helpful.
    """

    model = genai.GenerativeModel(
        model_name='gemini-2.0-flash-exp',
        tools=gemini_tools,
        system_instruction=system_instruction
    )

    # The history object must be a list of dicts in the format:
    # [{'role': 'user', 'parts': ['...']}, {'role': 'model', 'parts': ['...']}]
    # The SDK can consume this directly.
    chat = model.start_chat(history=history)
    
    response = await chat.send_message_async(message)
    
    # Check for function calls
    if response.parts and response.parts[0].function_call:
        fc = response.parts[0].function_call
        return {
            "text": None,
            "tool_call": {
                "name": fc.name,
                "args": dict(fc.args)
            }
        }
    
    return {"text": response.text, "tool_call": None}


async def normalize_events(raw_html: str, source_url: str) -> List[Dict]:
    """
    Uses Gemini 2.0 Flash to extract structured event data from raw HTML.
    """
    prompt = f"""
    Extract events from the following HTML from {source_url}.
    Return a JSON list of objects with keys: title, venue, start_time (ISO), price_min, price_max, description, image_url.
    
    HTML:
    {raw_html[:30000]}  # Truncating to keep prompts manageable, though Flash has a 1M context window
    """

    response = await json_model.generate_content_async(prompt)
    try:
        return json.loads(response.text)
    except json.JSONDecodeError:
        # If the model output is not valid JSON, return an empty list.
        return []