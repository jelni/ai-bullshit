# syntax=docker/dockerfile:1
FROM python:3.12-bookworm

RUN apt-get update && \
    apt-get install -y --no-install-recommends git curl ca-certificates build-essential pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
RUN pip install --no-cache-dir "mistralai>=3.6.0" "mistralai-workflows[mistralai]==3.20.0" "pydantic>=2.14.1"
COPY src ./src
ENV PYTHONPATH=/app/src \
    HEALTH_SERVER_HOST=127.0.0.1 \
    HEALTH_SERVER_PORT=3001

CMD ["python", "-m", "daily_dev.entrypoint"]
