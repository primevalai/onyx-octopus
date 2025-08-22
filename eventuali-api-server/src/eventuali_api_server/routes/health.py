"""Health check routes for the API server."""

import logging
from datetime import datetime

from fastapi import APIRouter, HTTPException

from ..dependencies.database import db_manager
from ..models.events import HealthResponse

logger = logging.getLogger(__name__)

router = APIRouter(prefix="/health", tags=["health"])


@router.get("/", response_model=HealthResponse)
async def health_check() -> HealthResponse:
    """Check the health of the API server and its dependencies."""
    try:
        # Check database connection
        database_healthy = await db_manager.health_check()
        
        overall_status = "healthy" if database_healthy else "unhealthy"
        
        return HealthResponse(
            status=overall_status,
            timestamp=datetime.utcnow(),
            api_connected=True,
            database_connected=database_healthy
        )
        
    except Exception as e:
        logger.error(f"Health check failed: {e}")
        raise HTTPException(
            status_code=503,
            detail="Service unavailable"
        )


@router.get("/database")
async def database_health():
    """Check database connection health."""
    try:
        healthy = await db_manager.health_check()
        return {
            "status": "healthy" if healthy else "unhealthy",
            "timestamp": datetime.utcnow(),
            "connected": healthy
        }
    except Exception as e:
        logger.error(f"Database health check failed: {e}")
        raise HTTPException(
            status_code=503,
            detail=f"Database health check failed: {str(e)}"
        )