import chromadb
from llama_index.llms.ollama import Ollama
from llama_index.embeddings.huggingface import HuggingFaceEmbedding
from llama_index.core import (Settings, VectorStoreIndex, SimpleDirectoryReader, PromptTemplate)
from llama_index.core import StorageContext
from llama_index.vector_stores.chroma import ChromaVectorStore
from typing import Generator
import logging
import json
from app.models.chat import ChatRequest
from datetime import datetime, timezone
import time
import re
from app.models.chat import ChatRequest
import nltk
from nltk.tokenize import sent_tokenize

nltk.download('punkt_tab')

logger = logging.getLogger(__name__)

class LLMService:
    query_engine = None
    embed_model = None
    index = None

    @classmethod
    def initialize(cls):
        cls.embed_model = cls.init_embed_model()
        cls.index = cls.init_index(cls.embed_model)

    @staticmethod
    def init_embed_model():
        embed_model = HuggingFaceEmbedding(model_name="BAAI/bge-small-en-v1.5")
        return embed_model

    
    @staticmethod
    def init_index(embed_model):
        reader = SimpleDirectoryReader(input_dir="./docs", recursive=True)
        documents = reader.load_data()
        logger.info(f"Index creating with {len(documents)} documents")

        chroma_client = chromadb.EphemeralClient()
        chroma_collection = chroma_client.create_collection("iollama")
        vector_store = ChromaVectorStore(chroma_collection=chroma_collection)
        storage_context = StorageContext.from_defaults(vector_store=vector_store)

        return VectorStoreIndex.from_documents(documents, storage_context=storage_context, embed_model=embed_model)

    @staticmethod
    def chunk_text(text):
        sentences = sent_tokenize(text)
        chunks = []
        current_chunk = ""

        for sentence in sentences:
            if len(current_chunk) + len(sentence) > 150:  # Adjust this threshold as needed
                if current_chunk:
                    chunks.append(current_chunk.strip())
                current_chunk = sentence
            else:
                current_chunk += " " + sentence if current_chunk else sentence

        if current_chunk:
            chunks.append(current_chunk.strip())

        return chunks

    @classmethod
    def get_query_engine(cls, model: str):
        llm = Ollama(model=model, request_timeout=300.0)
        Settings.llm = llm
        Settings.embed_model = cls.embed_model

        template = (
            "Imagine you are a professional Rust developer and expert in low level programming, with access to all current and relevant rustbook, "
            "case studies, and expert analyses. Your goal is to provide insightful, accurate, and concise answers to questions in this domain.\n\n"
            "Here is some context related to the query:\n"
            "-----------------------------------------\n"
            "{context_str}\n"
            "-----------------------------------------\n"
            "Considering the above information, please respond to the following inquiry with detailed references to applicable principles where appropriate:\n\n"
            "Question: {query_str}\n\n"
            "Answer succinctly, starting with the phrase 'According to rustlang best practice,' and ensure your response is understandable to someone without low lever programming background."
        )

        qa_template = PromptTemplate(template)

        return cls.index.as_query_engine(text_qa_template=qa_template, similarity_top_k=3)

    @classmethod
    def generate_chat_response(cls, request: ChatRequest) -> Generator[str, None, None]:
        logger.info(f"Processing chat request for model: {request.model}")

        try:
            query_engine = cls.get_query_engine(request.model)

            question = " ".join([msg.content for msg in request.messages if msg.role == "user"])

            start_time = time.time()

            response = query_engine.query(question)

            end_time = time.time()
            total_duration = int((end_time - start_time) * 1e9)

            load_duration = int(total_duration * 0.1)
            prompt_eval_count = len(question.split())
            prompt_eval_duration = int(total_duration * 0.3)
            eval_count = len(response.response.split())
            eval_duration = int(total_duration * 0.6)

            logger.info(f"Got response from LLM: {response}")

            chunks = cls.chunk_text(response.response)

            for i, chunk in enumerate(chunks):
                response_chunk = {
                    "model": request.model,
                    "created_at": datetime.now(timezone.utc).isoformat(),
                    "message": {
                        "role": "assistant",
                        "content": chunk
                    },
                    "done": False
                }

                if i == len(chunks) - 1:
                    response_chunk["done"] = True
                    response_chunk["done_reason"] = "stop"

                    response_chunk.update({
                        "total_duration": total_duration,
                        "load_duration": load_duration,
                        "prompt_eval_count": prompt_eval_count,
                        "prompt_eval_duration": prompt_eval_duration,
                        "eval_count": eval_count,
                        "eval_duration": eval_duration,
                    })

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

