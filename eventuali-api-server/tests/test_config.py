"""Tests for configuration management."""

import os
import pytest

from eventuali_api_server.config import APIServerConfig, get_config, set_config


def test_default_config():
    """Test default configuration values."""
    config = APIServerConfig()
    
    assert config.host == "127.0.0.1"
    assert config.port == 8765
    assert config.reload is False
    assert config.log_level == "info"
    assert config.data_dir == ".events"
    assert config.database_timeout == 10.0
    assert config.title == "Eventuali API Server"
    assert config.version == "0.1.0"


def test_cors_defaults():
    """Test CORS default values are set correctly."""
    config = APIServerConfig()
    
    expected_origins = [
        "http://localhost:3210",
        "http://127.0.0.1:3210", 
        "http://localhost:3000",
        "http://127.0.0.1:3000",
    ]
    
    assert config.cors_origins == expected_origins
    assert config.cors_allow_credentials is True
    assert "GET" in config.cors_allow_methods
    assert "POST" in config.cors_allow_methods
    assert "*" in config.cors_allow_headers


def test_from_env(monkeypatch):
    """Test configuration from environment variables."""
    # Set environment variables
    monkeypatch.setenv("HOST", "0.0.0.0")
    monkeypatch.setenv("PORT", "9000")
    monkeypatch.setenv("RELOAD", "true")
    monkeypatch.setenv("LOG_LEVEL", "DEBUG")
    monkeypatch.setenv("DATA_DIR", "/tmp/events")
    monkeypatch.setenv("DATABASE_TIMEOUT", "20.0")
    monkeypatch.setenv("CORS_ORIGINS", "http://example.com,https://app.example.com")
    monkeypatch.setenv("CORS_ALLOW_CREDENTIALS", "false")
    monkeypatch.setenv("API_TITLE", "Test API")
    monkeypatch.setenv("API_VERSION", "1.0.0")
    
    config = APIServerConfig.from_env()
    
    assert config.host == "0.0.0.0"
    assert config.port == 9000
    assert config.reload is True
    assert config.log_level == "debug"
    assert config.data_dir == "/tmp/events"
    assert config.database_timeout == 20.0
    assert config.cors_origins == ["http://example.com", "https://app.example.com"]
    assert config.cors_allow_credentials is False
    assert config.title == "Test API"
    assert config.version == "1.0.0"


def test_global_config():
    """Test global configuration management."""
    # Create custom config
    custom_config = APIServerConfig(
        host="custom.host",
        port=1234,
        title="Custom API"
    )
    
    # Set global config
    set_config(custom_config)
    
    # Get global config
    retrieved_config = get_config()
    
    assert retrieved_config.host == "custom.host"
    assert retrieved_config.port == 1234
    assert retrieved_config.title == "Custom API"


def test_empty_cors_origins_env(monkeypatch):
    """Test empty CORS origins environment variable."""
    monkeypatch.setenv("CORS_ORIGINS", "")
    
    config = APIServerConfig.from_env()
    
    # Should use defaults when empty
    expected_origins = [
        "http://localhost:3210",
        "http://127.0.0.1:3210",
        "http://localhost:3000", 
        "http://127.0.0.1:3000",
    ]
    
    assert config.cors_origins == expected_origins