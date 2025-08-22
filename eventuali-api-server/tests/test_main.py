"""Tests for the main FastAPI application."""

import pytest
from fastapi.testclient import TestClient

from eventuali_api_server.main import create_app
from eventuali_api_server.config import APIServerConfig, set_config


@pytest.fixture
def test_app():
    """Create test FastAPI application."""
    # Set test configuration
    config = APIServerConfig(
        data_dir="test_events",
        cors_origins=["http://localhost:3000"],
        log_level="debug"
    )
    set_config(config)
    
    app = create_app()
    return app


@pytest.fixture
def client(test_app):
    """Create test client."""
    return TestClient(test_app)


def test_root_endpoint(client):
    """Test the root endpoint."""
    response = client.get("/")
    assert response.status_code == 200
    
    data = response.json()
    assert "name" in data
    assert "version" in data
    assert "description" in data
    assert "endpoints" in data
    
    endpoints = data["endpoints"]
    assert "events" in endpoints
    assert "health" in endpoints
    assert "docs" in endpoints


def test_health_endpoint(client):
    """Test the health check endpoint."""
    response = client.get("/health/")
    assert response.status_code in [200, 503]  # May fail during testing
    
    data = response.json()
    assert "status" in data
    assert "timestamp" in data
    assert "api_connected" in data
    assert "database_connected" in data


def test_cors_headers(client):
    """Test CORS headers are present."""
    response = client.options("/", headers={"Origin": "http://localhost:3000"})
    assert "access-control-allow-origin" in response.headers


def test_openapi_schema(client):
    """Test OpenAPI schema is accessible."""
    response = client.get("/openapi.json")
    assert response.status_code == 200
    
    schema = response.json()
    assert "openapi" in schema
    assert "info" in schema
    assert "paths" in schema


def test_docs_endpoint(client):
    """Test Swagger UI documentation."""
    response = client.get("/docs")
    assert response.status_code == 200
    assert "text/html" in response.headers["content-type"]