# app/core/config.py
from pydantic_settings import BaseSettings
import os

class Settings(BaseSettings):
    PROJECT_NAME: str = "Ollama RAG Documents"
    PROJECT_VERSION: str = "1.0.0"
    ALLOWED_HOSTS: list = ["*"]
    INIT_INDEX: bool = os.getenv('INIT_INDEX', 'false').lower() == 'true'
    INDEX_PERSIST_DIRECTORY: str = os.getenv('INDEX_PERSIST_DIRECTORY', "./data/chromadb")
    INDEX_PERSIST_DIRECTORY_FAISS: str = os.getenv('INDEX_PERSIST_DIRECTORY_FAISS', "./data/faiss")
    MONGO_HOST: str = os.getenv('MONGO_HOST', '172.17.0.1')
    MONGO_PORT: int = int(os.getenv('MONGO_PORT', 27017))
    LOG_LEVEL: str = os.getenv('LOG_LEVEL', 'ERROR')
    LOG_FILE: str = os.getenv('LOG_FILE', 'logs/app.log')
    PORT: int = int(os.getenv('PORT', 8000))
    ALLOW_DANGEROUS_DESERIALIZATION: bool = os.getenv('ALLOW_DANGEROUS_DESERIALIZATION', 'false').lower() == 'true'

settings = Settings()
