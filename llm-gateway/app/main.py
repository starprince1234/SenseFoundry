from fastapi import FastAPI

app = FastAPI(title="SenseFoundry LLM Gateway", version="0.1.0")


@app.get("/health")
async def health():
    return {"status": "ok", "llm_api_reachable": False}
