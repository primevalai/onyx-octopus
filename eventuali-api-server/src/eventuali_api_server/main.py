"""Main FastAPI application for the Eventuali API server."""

import logging
from contextlib import asynccontextmanager

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from .config import get_config
from .dependencies.database import db_manager
from .routes import events
from .routes.health import router as health_router

logger = logging.getLogger(__name__)


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan events."""
    config = get_config()
    
    # Startup
    logger.info(f"Starting Eventuali API Server v{config.version}")
    logger.info(f"Database directory: {config.data_dir}")
    
    # Initialize database connection
    await db_manager.get_store()
    logger.info("Database connection initialized")
    
    yield
    
    # Shutdown
    logger.info("Shutting down Eventuali API Server")
    await db_manager.close()
    logger.info("Database connection closed")


def create_app() -> FastAPI:
    """Create and configure the FastAPI application."""
    config = get_config()
    
    app = FastAPI(
        title=config.title,
        description=config.description,
        version=config.version,
        lifespan=lifespan
    )
    
    # Add CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=config.cors_origins,
        allow_credentials=config.cors_allow_credentials,
        allow_methods=config.cors_allow_methods,
        allow_headers=config.cors_allow_headers,
    )
    
    # Include routers
    app.include_router(events.router)
    app.include_router(health_router)
    
    # Root endpoint
    @app.get("/")
    async def root():
        """Root endpoint with basic API information."""
        return {
            "name": config.title,
            "version": config.version,
            "description": config.description,
            "endpoints": {
                "events": "/events",
                "health": "/health",
                "docs": "/docs",
                "openapi": "/openapi.json"
            }
        }
    
    return app


app = create_app()