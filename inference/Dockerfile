FROM python:3.11-slim AS builder
WORKDIR /build

COPY requirements.txt .
RUN pip wheel --no-cache-dir --wheel-dir /wheels \
        --index-url https://download.pytorch.org/whl/cpu torch==2.7.1 \
    && pip wheel --no-cache-dir --wheel-dir /wheels -r requirements.txt

FROM python:3.11-slim AS model-cache
RUN mkdir -p /models

FROM python:3.11-slim AS runtime
WORKDIR /app

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    INFER_DEVICE=cpu \
    MODEL_CACHE_DIR=/models

RUN apt-get update \
    && apt-get install --no-install-recommends --yes curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /wheels /wheels
COPY requirements.txt .
RUN pip install --no-cache-dir --no-index --find-links=/wheels torch==2.7.1 \
    && pip install --no-cache-dir --no-index --find-links=/wheels -r requirements.txt \
    && rm -rf /wheels

COPY --from=model-cache /models /models
COPY app/ ./inference/app/

EXPOSE 8000
CMD ["uvicorn", "inference.app.main:app", "--host", "0.0.0.0", "--port", "8000"]
