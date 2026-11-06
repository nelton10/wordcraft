# AI writing assistant

The native WordCraft desktop app includes a docked chat panel beside the document. Choose **Chat IA**
for Ollama or an API provider, or **DeepSeek Web** to use the DeepSeek website in the same panel.
Review generated API responses and insert them into the open document, or use **Generate and insert
into Word** to add them directly. In DeepSeek Web, copy the response and choose **Colar no documento**.
If no document is open, WordCraft creates a blank document first. Insertions are normal document edits
and can be undone.

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

On Windows, **DeepSeek Web** loads `https://chat.deepseek.com/` inside the docked panel, not in a
separate window. Sign in there; the WebView profile is kept in WordCraft's settings folder so the
login can persist between launches. This uses the DeepSeek website, not its API. Copy a response and
choose **Colar no documento** to insert it into the current WordCraft document. The API and Ollama
chat use WordCraft's native controls in the same docked panel. On other desktop platforms, DeepSeek
is available through an OpenAI-compatible API endpoint.

## Split long Ollama responses

For Ollama, enable **Dividir resposta para o Ollama** and choose 2–8 parts. WordCraft sends those
requests one at a time and combines the results into one preview. Each part has a lower output-token
limit, which can help on computers that struggle with long generations. The setting affects Ollama;
other providers continue to use one request per response.

Cloud provider keys are sent only to the selected provider over HTTPS. API usage and billing follow
the account and terms of that provider. The compatible endpoint setting is intended for trusted
services; the key is sent to that configured endpoint.

The assistant is available in the native desktop app; the web and CLI apps do not use this setup.
