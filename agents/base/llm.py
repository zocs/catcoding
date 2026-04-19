"""LLM integration for CatCoding agents.

OpenAI-compatible client — works with OpenAI, Anthropic (via proxy),
local models (ollama, vLLM), and any /v1/chat/completions endpoint.

Configuration (env vars):
    LLM_API_BASE  — base URL (e.g. https://api.openai.com/v1 or http://localhost:11434/v1)
    LLM_API_KEY   — API key (can be empty for local models)
    LLM_MODEL     — model name (e.g. gpt-4o, claude-3-5-sonnet, qwen2.5-coder)

If any var is missing, `is_available()` returns False and agents fall back
to scaffold mode.
"""

from __future__ import annotations

import json
import os
import urllib.error
import urllib.request
from typing import Optional


def is_available() -> bool:
    """Check if LLM env vars are configured."""
    return bool(os.environ.get("LLM_API_BASE") and os.environ.get("LLM_MODEL"))


def _api_base() -> str:
    return os.environ["LLM_API_BASE"].rstrip("/")


def _api_key() -> str:
    return os.environ.get("LLM_API_KEY", "")


def _model() -> str:
    return os.environ["LLM_MODEL"]


def chat(
    messages: list[dict],
    temperature: float = 0.2,
    max_tokens: int = 4096,
    timeout: int = 120,
) -> str:
    """Send a chat completion request and return the assistant content.

    Args:
        messages: [{"role": "system"|"user"|"assistant", "content": "..."}]
        temperature: sampling temperature (0 = deterministic)
        max_tokens: max response tokens
        timeout: HTTP timeout in seconds

    Returns:
        The assistant's response text.

    Raises:
        RuntimeError: on API errors.
    """
    url = f"{_api_base()}/chat/completions"
    body = {
        "model": _model(),
        "messages": messages,
        "temperature": temperature,
        "max_tokens": max_tokens,
    }
    data = json.dumps(body).encode("utf-8")

    headers = {"Content-Type": "application/json"}
    api_key = _api_key()
    if api_key:
        headers["Authorization"] = f"Bearer {api_key}"

    req = urllib.request.Request(url, data=data, headers=headers, method="POST")

    try:
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            result = json.loads(resp.read().decode("utf-8"))
    except urllib.error.HTTPError as e:
        error_body = e.read().decode("utf-8", errors="replace")
        raise RuntimeError(
            f"LLM API error {e.code}: {error_body[:500]}"
        ) from e
    except Exception as e:
        raise RuntimeError(f"LLM request failed: {e}") from e

    try:
        return result["choices"][0]["message"]["content"]
    except (KeyError, IndexError) as e:
        raise RuntimeError(
            f"Unexpected LLM response format: {json.dumps(result)[:500]}"
        ) from e


def generate_code(
    role: str,
    task_summary: str,
    task_description: str = "",
    file_path: str = "",
    workdir: str = ".",
) -> str:
    """Generate code for a task using the LLM.

    Returns the generated source code as a string.
    """
    # Read existing files for context (if any)
    context_files = []
    if file_path and os.path.exists(os.path.join(workdir, file_path)):
        try:
            with open(os.path.join(workdir, file_path), "r") as f:
                context_files.append(f"--- {file_path} ---\n{f.read()[:3000]}")
        except Exception:
            pass

    context_block = "\n\n".join(context_files) if context_files else "(empty project)"

    system_prompt = f"""You are a {role} developer on the CatCoding team.
Generate production-ready code for the given task.
Output ONLY the source code — no markdown fences, no explanations.
If creating a new file, output the complete file content.
If modifying an existing file, output the complete modified file."""

    user_prompt = f"""Task: {task_summary}

{f"Description: {task_description}" if task_description else ""}

{f"Target file: {file_path}" if file_path else ""}

Existing context:
{context_block}

Generate the code now."""

    return chat(
        [
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": user_prompt},
        ],
        temperature=0.2,
        max_tokens=4096,
    )
