# # app/api/routes.py
# from fastapi import APIRouter, HTTPException, Depends
# from app.models.question import QuestionRequest
# from app.services.llm_service import LLMService
# from fastapi.responses import StreamingResponse
# import logging
#
# router = APIRouter()
# logger = logging.getLogger(__name__)
#
# @router.post("/question")
# async def post_question(request: QuestionRequest):
#     try:
#         return StreamingResponse(
#             LLMService.generate_response(request.question, request.user_id),
#             media_type="text/plain"
#         )
#     except Exception as e:
#         logger.error(f"Error processing question: {str(e)}", exc_info=True)
#         raise HTTPException(status_code=500, detail="Internal server error")

# app/api/routes.py
# from fastapi import APIRouter, HTTPException
# from app.models.chat import ChatRequest
# from app.services.llm_service import LLMService
# from fastapi.responses import StreamingResponse
# import logging
# import json
# from datetime import datetime, timezone
#
# router = APIRouter()
# logger = logging.getLogger(__name__)
#
# @router.post("/chat")
# async def chat(request: ChatRequest):
#     try:
#         return StreamingResponse(
#             LLMService.generate_chat_response(request),
#             media_type="text/event-stream"
#         )
#     except Exception as e:
#         logger.error(f"Error processing chat request: {str(e)}", exc_info=True)
#         raise HTTPException(status_code=500, detail="Internal server error")
#


# app/api/routes.py
from fastapi import APIRouter, HTTPException, Request
from app.models.chat import ChatRequest
from app.services.llm_service import LLMService
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
