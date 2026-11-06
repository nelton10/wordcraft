# AI writing assistant

The native WordCraft desktop app includes an assistant that can draft text and insert it into the
open document. For example, ask it to create a classroom test, review the generated preview, or use
**Generate and insert into Word** to add the result directly. If no document is open, WordCraft
creates a blank document first. The insertion is a normal document edit and can be undone.

## Connect at launch

At each app launch, enter an OpenAI API key in the masked field and choose **Use this session**.
The key is held in memory for that run only; WordCraft does not save it in `ui.json`, log it, or
include it in the repository. The key is sent over HTTPS to the OpenAI Responses API when you
submit a prompt, and usage is billed to the API account that owns the key. Close the app and enter
the key again next time. You can open **Configure key** from the assistant to replace it during
the current session.

You can skip the prompt and continue using WordCraft without the assistant. The default model is
`gpt-6-luna`. The assistant is available in the native desktop app; the web and CLI apps do not
use this key.
