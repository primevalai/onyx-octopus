# Multi-stage build for eventuali-api-server
FROM python:3.12-slim as builder

# Install UV for fast Python package management
RUN pip install uv

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY pyproject.toml uv.lock* ./

# Install dependencies in virtual environment
RUN uv venv --python 3.12 /opt/venv && \
    . /opt/venv/bin/activate && \
    uv pip install -e . && \
    chmod -R 755 /opt/venv

# Production stage
FROM python:3.12-slim

# Install curl for health checks
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl && \
    rm -rf /var/lib/apt/lists/*

# Copy virtual environment from builder
COPY --from=builder /opt/venv /opt/venv

# Set up environment
ENV PATH="/opt/venv/bin:$PATH"
ENV PYTHONPATH="/app/src"

# Create non-root user for security
RUN groupadd -r eventuali && \
    useradd -r -g eventuali -d /app -s /bin/bash eventuali

# Set working directory
WORKDIR /app

# Copy application code
COPY --chown=eventuali:eventuali . .

# Create events directory for database
RUN mkdir -p /app/.events && \
    chown -R eventuali:eventuali /app/.events

# Switch to non-root user
USER eventuali

# Expose port
EXPOSE 8000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health/ || exit 1

# Default command using virtual environment python
CMD ["/opt/venv/bin/python", "-m", "eventuali_api_server.cli", "--host", "0.0.0.0", "--port", "8000"]