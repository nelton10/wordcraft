//! In-app writing assistant for the native desktop app.
//!
//! Provider credentials are requested at launch, kept only in memory, and never written to
//! preferences or logs. Ollama runs locally; cloud providers receive prompts over HTTPS.

use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use serde_json::{Value, json};
use wordcraft_ui_egui::WordApp;
use zeroize::{Zeroize, Zeroizing};

const MAX_PROMPT_CHARS: usize = 10_000;
const MAX_OUTPUT_CHARS: usize = 40_000;
const INSTRUCTIONS: &str = concat!(
    "Você é o assistente de redação integrado ao WordCraft. Responda em português brasileiro, a menos que o usuário peça outro idioma. ",
    "Quando pedirem uma prova ou atividade, produza um material pronto para uso, com título, identificação, instruções, questões numeradas ",
    "e alternativas quando fizer sentido. Escreva em texto simples, com parágrafos, sem tabelas Markdown nem cercas de código. ",
    "Não diga que inseriu o texto no documento; o aplicativo fará isso."
);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Provider {
    Ollama,
    Gemini,
    Claude,
    OpenAiCompatible,
}

impl Provider {
    const ALL: [Self; 4] = [Self::Ollama, Self::Gemini, Self::Claude, Self::OpenAiCompatible];

    fn label(self) -> &'static str {
        match self {
            Self::Ollama => "Ollama local",
            Self::Gemini => "Google Gemini",
            Self::Claude => "Anthropic Claude",
            Self::OpenAiCompatible => "API compatível (OpenAI, OpenRouter, Groq, DeepSeek…)",
        }
    }

    fn default_model(self) -> &'static str {
        match self {
            Self::Ollama => "qwen3:8b",
            Self::Gemini => "gemini-3.8-flash",
            Self::Claude => "claude-sonnet-4-6",
            Self::OpenAiCompatible => "gpt-4.1-mini",
        }
    }

    fn default_endpoint(self) -> &'static str {
        match self {
            Self::Ollama => "http://localhost:11434/v1",
            Self::OpenAiCompatible => "https://api.openai.com/v1",
            _ => "",
        }
    }

    fn needs_key(self) -> bool {
        matches!(self, Self::Gemini | Self::Claude)
    }

    fn endpoint_is_editable(self) -> bool {
        matches!(self, Self::Ollama | Self::OpenAiCompatible)
    }
}

struct RequestConfig {
    provider: Provider,
    model: String,
    endpoint: String,
    api_key: Option<Zeroizing<String>>,
}

pub struct AiAgent {
    config_dialog_open: bool,
    provider: Provider,
    model: String,
    endpoint: String,
    key_input: Zeroizing<String>,
    api_key: Option<Zeroizing<String>>,
    window_open: bool,
    prompt: String,
    generated: Option<String>,
    status: String,
    pending: Option<Receiver<Result<String, String>>>,
    insert_when_ready: bool,
}

impl Default for AiAgent {
    fn default() -> Self {
        let provider = Provider::Ollama;
        Self {
            config_dialog_open: false,
            provider,
            model: provider.default_model().to_string(),
            endpoint: provider.default_endpoint().to_string(),
            key_input: Zeroizing::new(String::new()),
            api_key: None,
            window_open: false,
            prompt: String::new(),
            generated: None,
            status: String::new(),
            pending: None,
            insert_when_ready: false,
        }
    }
}

impl AiAgent {
    /// Opens provider setup at every app launch. Credentials are not persisted.
    pub fn new() -> Self {
        Self { config_dialog_open: true, window_open: true, ..Self::default() }
    }

    pub fn show(&mut self, ctx: &egui::Context, app: &mut WordApp) {
        self.receive_result(app);
        if self.config_dialog_open {
            self.show_config_dialog(ctx);
        }
        if self.window_open {
            let mut open = self.window_open;
            egui::Window::new("Assistente de IA")
                .open(&mut open)
                .default_size([460.0, 520.0])
                .resizable(true)
                .show(ctx, |ui| self.show_chat(ui, app));
            self.window_open = open;
        } else {
            egui::Area::new(egui::Id::new("wordcraft.ai.open"))
                .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -36.0))
                .show(ctx, |ui| {
                    if ui.button("✦ Assistente IA").clicked() {
                        self.window_open = true;
                    }
                });
        }
    }

    fn show_config_dialog(&mut self, ctx: &egui::Context) {
        let mut close = false;
        egui::Modal::new(egui::Id::new("wordcraft.ai.provider")).show(ctx, |ui| {
            ui.set_min_width(420.0);
            ui.heading("Conectar assistente de IA");
            ui.label("Escolha um modelo local ou conecte uma API externa.");
            ui.add_space(8.0);

            let previous_provider = self.provider;
            egui::ComboBox::from_id_salt("wordcraft.ai.provider.select")
                .selected_text(self.provider.label())
                .show_ui(ui, |ui| {
                    for provider in Provider::ALL {
                        ui.selectable_value(&mut self.provider, provider, provider.label());
                    }
                });
            if previous_provider != self.provider {
                self.model = self.provider.default_model().to_string();
                self.endpoint = self.provider.default_endpoint().to_string();
                self.key_input.zeroize();
                self.api_key = None;
            }

            ui.add_space(8.0);
            ui.label("Modelo");
            ui.add(egui::TextEdit::singleline(&mut self.model).desired_width(f32::INFINITY));
            if self.provider.endpoint_is_editable() {
                ui.add_space(6.0);
                ui.label(if self.provider == Provider::Ollama { "Endereço do Ollama" } else { "Endereço base da API" });
                ui.add(egui::TextEdit::singleline(&mut self.endpoint).desired_width(f32::INFINITY));
            }

            if self.provider.needs_key() || self.provider == Provider::OpenAiCompatible {
                ui.add_space(6.0);
                ui.label(if self.provider.needs_key() { "Chave da API" } else { "Chave da API (se exigida pelo serviço)" });
                ui.add(
                    egui::TextEdit::singleline(&mut self.key_input)
                        .password(true)
                        .desired_width(f32::INFINITY)
                        .hint_text("A chave fica apenas nesta sessão"),
                );
            }
            ui.add_space(6.0);
            let note = match self.provider {
                Provider::Ollama => "O Ollama precisa estar instalado e em execução. O modelo deve estar baixado no computador.",
                Provider::Gemini => "A solicitação será enviada ao Google Gemini usando sua chave de API.",
                Provider::Claude => "A solicitação será enviada à Anthropic usando sua chave de API.",
                Provider::OpenAiCompatible => "Use um endpoint compatível com /chat/completions, como OpenAI, OpenRouter, Groq ou DeepSeek.",
            };
            ui.small(note);
            ui.small("Chaves não são salvas nas preferências, nos documentos ou no repositório.");
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Conectar nesta sessão").clicked() {
                    if self.model.trim().is_empty() {
                        self.status = "Informe o nome do modelo.".into();
                    } else if self.provider.endpoint_is_editable() && self.endpoint.trim().is_empty() {
                        self.status = "Informe o endereço da API.".into();
                    } else if self.provider.needs_key() && self.key_input.trim().is_empty() {
                        self.status = "Digite a chave da API para este provedor.".into();
                    } else {
                        self.api_key = (!self.key_input.trim().is_empty())
                            .then(|| Zeroizing::new(self.key_input.trim().to_string()));
                        self.key_input.zeroize();
                        self.model = self.model.trim().to_string();
                        self.endpoint = self.endpoint.trim().trim_end_matches('/').to_string();
                        self.status = format!("{} configurado para esta sessão.", self.provider.label());
                        close = true;
                    }
                }
                if ui.button("Agora não").clicked() {
                    self.key_input.zeroize();
                    close = true;
                }
            });
        });
        if close {
            self.config_dialog_open = false;
        }
    }

    fn show_chat(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        ui.horizontal(|ui| {
            ui.label(format!("{} · {}", self.provider.label().trim(), self.model));
            if ui.button("Configurar IA").clicked() {
                self.key_input.zeroize();
                self.config_dialog_open = true;
            }
        });
        ui.label("Peça para redigir ou organizar conteúdo. O resultado pode ser inserido no documento aberto.");
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::multiline(&mut self.prompt)
                .desired_rows(4)
                .hint_text("Ex.: Crie uma prova de matemática para o 1º ano, com 10 questões sobre porcentagem e gabarito."),
        );
        ui.horizontal(|ui| {
            let disabled = self.pending.is_some() || !self.is_configured() || self.prompt.trim().is_empty();
            if ui.add_enabled(!disabled, egui::Button::new("Gerar resposta")).clicked() {
                self.start_request(false, ui.ctx());
            }
            if ui.add_enabled(!disabled, egui::Button::new("Gerar e inserir no Word")).clicked() {
                self.start_request(true, ui.ctx());
            }
        });
        if self.pending.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Preparando o texto…");
            });
        }
        if !self.status.is_empty() {
            ui.small(&self.status);
        }
        let mut insert_preview = false;
        if let Some(text) = self.generated.as_ref() {
            ui.separator();
            ui.label("Prévia da resposta");
            egui::ScrollArea::vertical().max_height(210.0).show(ui, |ui| ui.label(text));
            insert_preview = ui.button("Inserir prévia no documento").clicked();
        }
        if insert_preview && let Some(text) = self.generated.clone() {
            self.insert_into_document(app, text);
        }
    }

    fn is_configured(&self) -> bool {
        !self.model.trim().is_empty()
            && (!self.provider.endpoint_is_editable() || !self.endpoint.trim().is_empty())
            && (!self.provider.needs_key() || self.api_key.is_some())
    }

    fn start_request(&mut self, insert_when_ready: bool, ctx: &egui::Context) {
        if !self.is_configured() {
            self.status = "Configure o modelo e a chave (se necessária) antes de enviar.".into();
            self.config_dialog_open = true;
            return;
        }
        let prompt = self.prompt.trim();
        if prompt.is_empty() {
            self.status = "Digite o que deseja criar.".into();
            return;
        }
        if prompt.chars().count() > MAX_PROMPT_CHARS {
            self.status = format!("A solicitação deve ter até {MAX_PROMPT_CHARS} caracteres.");
            return;
        }
        let config = RequestConfig {
            provider: self.provider,
            model: self.model.clone(),
            endpoint: self.endpoint.clone(),
            api_key: self.api_key.as_ref().map(|key| Zeroizing::new(key.as_str().to_string())),
        };
        let prompt = prompt.to_string();
        let (tx, rx) = channel();
        let repaint = ctx.clone();
        std::thread::spawn(move || {
            let result = request_model(&config, &prompt);
            let _ = tx.send(result);
            repaint.request_repaint();
        });
        self.pending = Some(rx);
        self.insert_when_ready = insert_when_ready;
        self.generated = None;
        self.status = format!("Enviando solicitação para {}…", self.provider.label().trim());
    }

    fn receive_result(&mut self, app: &mut WordApp) {
        let Some(receiver) = self.pending.as_ref() else { return };
        let Ok(result) = receiver.try_recv() else { return };
        self.pending = None;
        match result {
            Ok(text) => {
                self.generated = Some(text.clone());
                if self.insert_when_ready {
                    self.insert_into_document(app, text);
                } else {
                    self.status = "Resposta pronta. Revise e escolha se deseja inserir no documento.".into();
                }
            }
            Err(error) => self.status = error,
        }
        self.insert_when_ready = false;
    }

    fn insert_into_document(&mut self, app: &mut WordApp, text: String) {
        if text.trim().is_empty() {
            self.status = "A resposta está vazia e não foi inserida.".into();
            return;
        }
        if !app.has_document() && let Err(error) = app.run("file.new", json!({"template": "blank"})) {
            self.status = format!("Não foi possível criar o documento: {error}");
            return;
        }
        match app.run("text.insert", json!({"text": text, "paragraphs": true})) {
            Ok(_) => self.status = "Conteúdo inserido no documento. Use Desfazer se quiser removê-lo.".into(),
            Err(error) => self.status = format!("Não foi possível inserir o conteúdo: {error}"),
        }
    }
}

fn request_model(config: &RequestConfig, prompt: &str) -> Result<String, String> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(90)))
        .build()
        .into();
    let (request, body) = match config.provider {
        Provider::Ollama | Provider::OpenAiCompatible => {
            let url = format!("{}/chat/completions", config.endpoint.trim_end_matches('/'));
            let request = agent.post(&url);
            let request = if let Some(key) = config.api_key.as_ref() {
                request.header("Authorization", &format!("Bearer {}", key.as_str()))
            } else if config.provider == Provider::Ollama {
                request.header("Authorization", "Bearer ollama")
            } else {
                request
            };
            let body = json!({
                "model": config.model,
                "messages": [{"role":"system", "content":INSTRUCTIONS}, {"role":"user", "content":prompt}],
                "max_tokens": 4000
            });
            (request, body)
        }
        Provider::Gemini => {
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                config.model
            );
            let key = config.api_key.as_ref().map(|key| key.as_str()).unwrap_or_default();
            let request = agent.post(&url).header("x-goog-api-key", key);
            let body = json!({
                "systemInstruction": {"parts": [{"text": INSTRUCTIONS}]},
                "contents": [{"role":"user", "parts": [{"text": prompt}]}],
                "generationConfig": {"maxOutputTokens": 4000}
            });
            (request, body)
        }
        Provider::Claude => {
            let url = "https://api.anthropic.com/v1/messages".to_string();
            let key = config.api_key.as_ref().map(|key| key.as_str()).unwrap_or_default();
            let request = agent
                .post(&url)
                .header("x-api-key", key)
                .header("anthropic-version", "2023-06-01");
            let body = json!({
                "model": config.model,
                "max_tokens": 4000,
                "system": INSTRUCTIONS,
                "messages": [{"role":"user", "content":prompt}]
            });
            (request, body)
        }
    };

    let mut response = request.send_json(body).map_err(|error| {
        // Do not include the request or headers in the error; they may contain a secret key.
        let _ = error;
        format!("Não foi possível chamar {}. Confira conexão, modelo, endereço e chave.", config.provider.label().trim())
    })?;
    let value: Value = response.body_mut().read_json()
        .map_err(|_| format!("{} retornou uma resposta que não pôde ser lida.", config.provider.label().trim()))?;
    let text = match config.provider {
        Provider::Ollama | Provider::OpenAiCompatible => value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| value.pointer("/choices/0/message/content/0/text").and_then(Value::as_str).map(str::to_owned)),
        Provider::Gemini => value
            .pointer("/candidates/0/content/parts")
            .and_then(Value::as_array)
            .map(|parts| parts.iter().filter_map(|part| part.get("text").and_then(Value::as_str)).collect::<Vec<_>>().join("\n")),
        Provider::Claude => value
            .get("content")
            .and_then(Value::as_array)
            .map(|parts| parts.iter().filter_map(|part| part.get("text").and_then(Value::as_str)).collect::<Vec<_>>().join("\n")),
    }
    .unwrap_or_default();
    if text.trim().is_empty() {
        let detail = value.pointer("/error/message").and_then(Value::as_str).unwrap_or("nenhum texto foi retornado");
        return Err(format!("A API não retornou texto: {detail}"));
    }
    if text.chars().count() > MAX_OUTPUT_CHARS {
        return Err("A resposta passou do limite de tamanho do WordCraft.".into());
    }
    Ok(text)
}
