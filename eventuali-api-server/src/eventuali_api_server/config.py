"""Configuration management for the API server."""

import os
from dataclasses import dataclass
from pathlib import Path
from typing import List, Optional


@dataclass
class APIServerConfig:
    """Configuration settings for the API server."""
    
    # Server settings
    host: str = "127.0.0.1"
    port: int = 8765
    reload: bool = False
    log_level: str = "info"
    
    # Database settings
    data_dir: str = ".events"
    database_timeout: float = 10.0
    
    # CORS settings
    cors_origins: List[str] = None
    cors_allow_credentials: bool = True
    cors_allow_methods: List[str] = None
    cors_allow_headers: List[str] = None
    
    # API settings
    title: str = "Eventuali API Server"
    description: str = "FastAPI server for Eventuali event sourcing system"
    version: str = "0.1.0"
    
    def __post_init__(self):
        """Set default values that depend on other values."""
        if self.cors_origins is None:
            self.cors_origins = [
                "http://localhost:3210",
                "http://127.0.0.1:3210",
                "http://localhost:3000",
                "http://127.0.0.1:3000",
            ]
        
        if self.cors_allow_methods is None:
            self.cors_allow_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
        
        if self.cors_allow_headers is None:
            self.cors_allow_headers = ["*"]
    
    @classmethod
    def from_env(cls) -> "APIServerConfig":
        """Create configuration from environment variables."""
        cors_origins = os.getenv("CORS_ORIGINS", "").strip()
        if cors_origins:
            cors_origins_list = [origin.strip() for origin in cors_origins.split(",")]
        else:
            cors_origins_list = None
        
        return cls(
            host=os.getenv("HOST", "127.0.0.1"),
            port=int(os.getenv("PORT", "8765")),
            reload=os.getenv("RELOAD", "false").lower() == "true",
            log_level=os.getenv("LOG_LEVEL", "info").lower(),
            data_dir=os.getenv("DATA_DIR", ".events"),
            database_timeout=float(os.getenv("DATABASE_TIMEOUT", "10.0")),
            cors_origins=cors_origins_list,
            cors_allow_credentials=os.getenv("CORS_ALLOW_CREDENTIALS", "true").lower() == "true",
            title=os.getenv("API_TITLE", "Eventuali API Server"),
            description=os.getenv("API_DESCRIPTION", "FastAPI server for Eventuali event sourcing system"),
            version=os.getenv("API_VERSION", "0.1.0"),
        )


# Global configuration instance
_config: Optional[APIServerConfig] = None


def get_config() -> APIServerConfig:
    """Get the global configuration instance."""
    global _config
    if _config is None:
        _config = APIServerConfig.from_env()
    return _config


def set_config(config: APIServerConfig) -> None:
    """Set the global configuration instance."""
    global _config
    _config = config