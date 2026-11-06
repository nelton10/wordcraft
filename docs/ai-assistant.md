# AI writing assistant

The native WordCraft desktop app includes an assistant that can draft text and insert it into the
open document. Ask it to create a classroom test, review the generated preview, or use **Generate and
insert into Word** to add the result directly. If no document is open, WordCraft creates a blank
document first. The insertion is a normal document edit and can be undone.

## Choose a provider at launch

The connection dialog appears whenever the desktop app starts. Select a provider, enter its model
name, and provide a key when the service requires one. Credentials are held in memory for that run
only and are not written to `ui.json`, logs, documents, or the repository. Close the app and set up
the provider again next time. You can open **Configure AI** in the assistant to change providers
during the current session.

| Provider | Setup |
|---|---|
| Ollama local | Install and start Ollama, download a model, then use its model name (default `qwen3:8b`). Default endpoint: `http://localhost:11434/v1`. Prompts are sent to the local Ollama server. |
| Google Gemini | Enter a Gemini model name (default `gemini-3.8-flash`) and a Google AI Studio API key. |
| Anthropic Claude | Enter a Claude model name (default `claude-sonnet-4-6`) and an Anthropic API key. |
| OpenAI-compatible API | Enter the model, base endpoint and optional key. This supports OpenAI and compatible services such as OpenRouter, Groq, and DeepSeek. |

## DeepSeek Web without an API key

Choose **Open DeepSeek Web** in the assistant header to open `https://chat.deepseek.com/` in the
system browser. Sign in to your DeepSeek account there and keep the chat beside WordCraft by placing
the browser window next to the app. This uses the DeepSeek website, not its API: prompts and replies
stay in the browser, so copy the response and paste it into WordCraft yourself. The site is not
embedded in the egui desktop window.

Cloud provider keys are sent only to the selected provider over HTTPS. API usage and billing follow
the account and terms of that provider. The compatible endpoint setting is intended for trusted
services; the key is sent to that configured endpoint.

The assistant is available in the native desktop app; the web and CLI apps do not use this setup.
