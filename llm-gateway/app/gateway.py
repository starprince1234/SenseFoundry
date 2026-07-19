import os
from collections.abc import Sequence

import httpx
from pydantic import BaseModel

PROMPT_TEMPLATE = (
    "Based ONLY on the following evidence, draft a definition for '{headword}' ({pos}). "
    "Evidence: {evidence_texts}. Return ONLY a definition without inventing examples."
)


class _Message(BaseModel):
    content: str


class _Choice(BaseModel):
    message: _Message


class _Completion(BaseModel):
    choices: list[_Choice]


class LlmGateway:
    def __init__(self, api_key: str | None = None, api_url: str | None = None) -> None:
        self.api_key: str | None = api_key or os.getenv("LLM_API_KEY")
        self.api_url: str = api_url or os.getenv(
            "LLM_API_URL", "https://api.openai.com/v1/chat/completions"
        )

    @staticmethod
    def build_prompt(headword: str, pos: str, evidence_texts: Sequence[str]) -> str:
        return PROMPT_TEMPLATE.format(
            headword=headword,
            pos=pos,
            evidence_texts="\n".join(evidence_texts),
        )

    async def draft_definition(
        self, headword: str, pos: str, evidence_texts: Sequence[str]
    ) -> str:
        if not self.api_key:
            raise RuntimeError("LLM_API_KEY is not configured")
        prompt = self.build_prompt(headword, pos, evidence_texts)
        payload = {
            "model": os.getenv("LLM_MODEL", "gpt-4o-mini"),
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0,
        }
        async with httpx.AsyncClient(timeout=15) as client:
            response = await client.post(
                self.api_url,
                headers={"Authorization": f"Bearer {self.api_key}"},
                json=payload,
            )
            _ = response.raise_for_status()
            body = _Completion.model_validate_json(response.content)
            if not body.choices:
                raise RuntimeError("LLM returned no completion choices")
            return body.choices[0].message.content.strip()

    async def reachable(self) -> bool:
        if not self.api_key:
            return False
        health_url = os.getenv("LLM_HEALTH_URL", self.api_url)
        try:
            async with httpx.AsyncClient(timeout=3) as client:
                response = await client.get(
                    health_url, headers={"Authorization": f"Bearer {self.api_key}"}
                )
            return response.status_code < 500
        except httpx.HTTPError:
            return False
