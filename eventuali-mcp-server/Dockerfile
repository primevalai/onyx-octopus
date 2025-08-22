# Multi-stage build for eventuali-mcp-server
FROM python:3.11-slim as builder

# Install UV for fast Python package management
RUN pip install uv

# Set working directory
WORKDIR /app

# Copy API server wheel and dependency files
COPY eventuali_api_server-0.1.0-py3-none-any.whl ./
COPY pyproject.toml uv.lock* ./

# Install dependencies in virtual environment
RUN uv venv /opt/venv && \
    . /opt/venv/bin/activate && \
    uv pip install eventuali_api_server-0.1.0-py3-none-any.whl && \
    uv pip install -e .

# Production stage
FROM python:3.11-slim

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

# Switch to non-root user
USER eventuali

# MCP servers communicate via stdio, so we need to ensure proper handling
# Set unbuffered Python output for stdio communication
ENV PYTHONUNBUFFERED=1

# Default command - runs the MCP server in stdio mode
CMD ["python", "-m", "eventuali_mcp_server"]