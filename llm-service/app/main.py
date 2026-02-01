from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from typing import Optional

# Initialize the app
app = FastAPI(title="Locate918 LLM Service")

class ChatRequest(BaseModel):
    message: str
    user_id: Optional[str] = None
    conversation_id: Optional[str] = None

class SearchRequest(BaseModel):
    query: str

# Setup CORS (This allows your React frontend to talk to this Python backend)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Allows all origins
    allow_credentials=True,
    allow_methods=["*"],  # Allows all methods
    allow_headers=["*"],  # Allows all headers
)

@app.get("/")
async def root():
    return {"status": "online", "service": "Locate918 LLM"}

@app.get("/health")
async def health_check():
    return {"status": "ok"}

@app.post("/api/chat")
async def chat(request: ChatRequest):
    return {
        "message": "Tully is online! (This is a placeholder response.)",
        "events": [],
        "conversation_id": request.conversation_id or "test"
    }

@app.post("/api/search")
async def search(request: SearchRequest):
    return {
        "events": [],
        "parsed": {"query": request.query}
    }