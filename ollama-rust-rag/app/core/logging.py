import logging
from logging.handlers import RotatingFileHandler
import os
from app.core.config import settings

def setup_logging():
    log_dir = os.path.dirname(settings.LOG_FILE)
    if not os.path.exists(log_dir):
        os.makedirs(log_dir)

    logging.basicConfig(level=settings.LOG_LEVEL)
    logger = logging.getLogger()

    file_handler = RotatingFileHandler(
        settings.LOG_FILE, maxBytes=10585787, backupCount=5
    )
    file_formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    file_handler.setFormatter(file_formatter)

    logger.addHandler(file_handler)
