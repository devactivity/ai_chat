# app/api/routes.py
from fastapi import APIRouter, HTTPException, Request
from app.models.chat import ChatRequest
from app.services.llm_service import LLMService
from app.services.langchain_service import LangchainService
from app.services.scrape_service import ScrapeService
from fastapi.responses import StreamingResponse
import logging
import json

router = APIRouter()
logger = logging.getLogger(__name__)

@router.post("/chat")
async def chat(request: Request):
    try:
        # Parse the raw JSON from the request body
        body = await request.body()
        chat_request = ChatRequest.parse_raw(body)
        
        return StreamingResponse(
            LLMService.generate_chat_response(chat_request),
            media_type="text/event-stream"
        )
    except json.JSONDecodeError:
        logger.error("Invalid JSON in request body")
        raise HTTPException(status_code=400, detail="Invalid JSON in request body")
    except Exception as e:
        logger.error(f"Error processing chat request: {str(e)}", exc_info=True)
        raise HTTPException(status_code=500, detail="Internal server error")

@router.post("/chit")
async def chit(request: Request):
    try:
        # Parse the raw JSON from the request body
        body = await request.body()
        chat_request = ChatRequest.parse_raw(body)
        
        return StreamingResponse(
            LangchainService.generate_chat_response(chat_request),
            media_type="text/event-stream"
        )
    except json.JSONDecodeError:
        logger.error("Invalid JSON in request body")
        raise HTTPException(status_code=400, detail="Invalid JSON in request body")
    except Exception as e:
        logger.error(f"Error processing chit request: {str(e)}", exc_info=True)
        raise HTTPException(status_code=500, detail="Internal server error")

@router.post("/scrape")
async def scrape(request: Request):
    try:
        body = await request.body()
        data = json.loads(body)
        target_url = data.get("url")
        chat_request = ChatRequest.parse_obj(data)

        if not target_url:
            raise HTTPException(status_code=400, detail="URL is required")

        # Initialize ScrapeService with the target URL
        await ScrapeService.initialize(target_url)

        return StreamingResponse(
            ScrapeService.generate_chat_response(chat_request),
            media_type="text/event-stream"
        )
    except json.JSONDecodeError:
        logger.error("Invalid JSON in request body")
        raise HTTPException(status_code=400, detail="Invalid JSON in request body")
    except Exception as e:
        logger.error(f"Error processing scrape request: {str(e)}", exc_info=True)
        raise HTTPException(status_code=500, detail="Internal server error")
