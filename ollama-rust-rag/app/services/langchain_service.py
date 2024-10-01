# app/services/langchain_service.py
from langchain_community.llms import Ollama
from langchain_huggingface import HuggingFaceEmbeddings
from langchain.text_splitter import RecursiveCharacterTextSplitter
from langchain_community.vectorstores import FAISS
from langchain.chains import ConversationalRetrievalChain
from langchain_community.document_loaders import DirectoryLoader
from app.models.chat import ChatRequest
from app.core.config import settings
import logging
import json
from datetime import datetime, timezone
import time
from typing import Generator
import os

logger = logging.getLogger(__name__)

class LangchainService:
    conversation = None
    vectorstore = None

    @classmethod
    async def initialize(cls):
        await cls.init_index()
        await cls.init_conversation()

    @classmethod
    async def init_index(cls):
        if not settings.INIT_INDEX:
            logger.info("Continuing without initializing index")
            return

        # Load documents from the docs directory
        loader = DirectoryLoader("./docs", recursive=True)
        documents = loader.load()
        
        logger.info(f"Index creating with {len(documents)} documents")

        # Split text
        text_splitter = RecursiveCharacterTextSplitter(chunk_size=1000, chunk_overlap=200)
        texts = text_splitter.split_documents(documents)

        # Create embeddings
        embeddings = HuggingFaceEmbeddings(model_name="sentence-transformers/all-MiniLM-L6-v2")

        # Create and save FAISS index
        cls.vectorstore = FAISS.from_documents(texts, embeddings)
        cls.vectorstore.save_local(settings.INDEX_PERSIST_DIRECTORY_FAISS)

    @classmethod
    async def init_conversation(cls):
        embeddings = HuggingFaceEmbeddings(model_name="sentence-transformers/all-MiniLM-L6-v2")
        
        if os.path.exists(settings.INDEX_PERSIST_DIRECTORY_FAISS):
            cls.vectorstore = FAISS.load_local(
                settings.INDEX_PERSIST_DIRECTORY_FAISS,
                embeddings,
                allow_dangerous_deserialization=settings.ALLOW_DANGEROUS_DESERIALIZATION
            )
        else:
            logger.warning("FAISS index not found. Initializing an empty vectorstore.")
            cls.vectorstore = FAISS.from_texts(["Placeholder"], embeddings)

        llm = Ollama(model="llama3.2:latest", base_url="http://localhost:11434")

        cls.conversation = ConversationalRetrievalChain.from_llm(
            llm,
            retriever=cls.vectorstore.as_retriever(),
            return_source_documents=True,
            verbose=True,
        )

    @classmethod
    def generate_chat_response(cls, request: ChatRequest) -> Generator[str, None, None]:
        logger.info(f"Processing chat request for model: {request.model}")

        try:
            question = " ".join([msg.content for msg in request.messages if msg.role == "user"])
            chat_history = []  # In a real application, you'd want to manage chat history

            start_time = time.time()

            response = cls.conversation({"question": question, "chat_history": chat_history})

            end_time = time.time()
            total_duration = int((end_time - start_time) * 1e9)

            answer = response['answer']
            logger.info(f"Got response from LLM: {answer}")

            response_chunk = {
                "model": request.model,
                "created_at": datetime.now(timezone.utc).isoformat(),
                "message": {
                    "role": "assistant",
                    "content": answer
                },
                "done": True,
                "done_reason": "stop",
                "total_duration": total_duration,
            }

            yield json.dumps(response_chunk) + "\n"

        except Exception as e:
            logger.error(f"Error generating chat response: {str(e)}", exc_info=True)
            
            error_chunk = {
                "model": request.model,
                "created_at": datetime.now(timezone.utc).isoformat(),
                "message": {
                    "role": "assistant",
                    "content": "An error occurred while processing your request. Please try again later."
                },
                "done": True,
                "done_reason": "error"
            }

            yield json.dumps(error_chunk) + "\n"

