from pydantic_settings import BaseSettings
import os

class Settings(BaseSettings):
    PROJECT_NAME: str = "Ollama RAG Documents"
    PROJECT_VERSION: str = "1.0.0"
    ALLOWED_HOSTS: list = ["*"]
    INIT_INDEX: bool = os.getenv('INIT_INDEX', 'false').lower() == 'true'
    INDEX_PERSIST_DIRECTORY: str = os.getenv('INDEX_PERSIST_DIRECTORY', "./data/chromadb")
    MONGO_HOST: str = os.getenv('MONGO_HOST', '172.17.0.1')
    MONGO_PORT: int = int(os.getenv('MONGO_PORT', 27017))
    LOG_LEVEL: str = os.getenv('LOG_LEVEL', 'ERROR')
    LOG_FILE: str = os.getenv('LOG_FILE', 'logs/app.log')

settings = Settings()
