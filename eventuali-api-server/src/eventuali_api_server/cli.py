"""Command-line interface for the Eventuali API server."""

import logging
import sys
from typing import Optional

import click
import uvicorn

from .config import APIServerConfig, set_config


def setup_logging(log_level: str) -> None:
    """Setup logging configuration."""
    level = getattr(logging, log_level.upper(), logging.INFO)
    logging.basicConfig(
        level=level,
        format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        datefmt='%Y-%m-%d %H:%M:%S'
    )


@click.command()
@click.option(
    "--host",
    default="127.0.0.1",
    help="Host to bind the server to",
    envvar="HOST"
)
@click.option(
    "--port",
    default=8765,
    type=int,
    help="Port to bind the server to",
    envvar="PORT"
)
@click.option(
    "--reload/--no-reload",
    default=False,
    help="Enable auto-reload for development",
    envvar="RELOAD"
)
@click.option(
    "--log-level",
    default="info",
    type=click.Choice(["debug", "info", "warning", "error", "critical"]),
    help="Logging level",
    envvar="LOG_LEVEL"
)
@click.option(
    "--data-dir",
    default=".events",
    help="Directory to store event data",
    envvar="DATA_DIR"
)
@click.option(
    "--cors-origins",
    help="Comma-separated list of CORS origins",
    envvar="CORS_ORIGINS"
)
@click.option(
    "--database-timeout",
    default=10.0,
    type=float,
    help="Database operation timeout in seconds",
    envvar="DATABASE_TIMEOUT"
)
def main(
    host: str,
    port: int,
    reload: bool,
    log_level: str,
    data_dir: str,
    cors_origins: Optional[str],
    database_timeout: float
) -> None:
    """Start the Eventuali API server."""
    
    # Setup logging
    setup_logging(log_level)
    logger = logging.getLogger(__name__)
    
    # Parse CORS origins
    cors_origins_list = None
    if cors_origins:
        cors_origins_list = [origin.strip() for origin in cors_origins.split(",")]
    
    # Create configuration
    config = APIServerConfig(
        host=host,
        port=port,
        reload=reload,
        log_level=log_level,
        data_dir=data_dir,
        database_timeout=database_timeout,
        cors_origins=cors_origins_list
    )
    
    # Set global configuration
    set_config(config)
    
    logger.info(f"Starting Eventuali API Server on {host}:{port}")
    logger.info(f"Data directory: {data_dir}")
    logger.info(f"Reload mode: {reload}")
    logger.info(f"Log level: {log_level}")
    
    try:
        uvicorn.run(
            "eventuali_api_server.main:app",
            host=host,
            port=port,
            reload=reload,
            log_level=log_level,
            access_log=True
        )
    except KeyboardInterrupt:
        logger.info("Server shutdown requested")
        sys.exit(0)
    except Exception as e:
        logger.error(f"Server startup failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()