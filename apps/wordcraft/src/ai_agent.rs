//! In-app writing assistant for the native desktop app.
//!
//! The API key is requested at launch and kept only in memory for this process. It is sent over
//! HTTPS directly to OpenAI and is never included in preferences, logs, or source files.

use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use serde_json::{Value, json};
use wordcraft_ui_egui::WordApp;
use zeroize::{Zeroize, Zeroizing};

const MODEL: &str = "gpt-6-luna";
const MAX_PROMPT_CHARS: usize = 10_000;
const MAX_OUTPUT_CHARS: usize = 40_000;
const INSTRUCTIONS: &str = concat!(
    "Você é o assistente de redação integrado ao WordCraft. Responda em português brasileiro, a menos que o usuário peça outro idioma. ",
    "Quando pedirem uma prova ou atividade, produza um material pronto para uso, com título, identificação, instruções, questões numeradas ",
    "e alternativas quando fizer sentido. Escreva em texto simples, com parágrafos, sem tabelas Markdown nem cercas de código. ",
    "Não diga que inseriu o texto no documento; o aplicativo fará isso."
);

pub struct AiAgent {
    key_dialog_open: bool,
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
        Self {
            key_dialog_open: false,
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
    /// Starts with a key-entry prompt every time the desktop app opens. No key is persisted.
    pub fn new() -> Self {
        Self { key_dialog_open: true, window_open: true, ..Self::default() }
    }

    pub fn show(&mut self, ctx: &egui::Context, app: &mut WordApp) {
        self.receive_result(app);
        if self.key_dialog_open {
            self.show_key_dialog(ctx);
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

    fn show_key_dialog(&mut self, ctx: &egui::Context) {
        let mut close = false;
        egui::Modal::new(egui::Id::new("wordcraft.ai.key")).show(ctx, |ui| {
            ui.set_min_width(380.0);
            ui.heading("Conectar assistente de IA");
            ui.label("Informe sua chave da API da OpenAI para usar o assistente nesta sessão.");
            ui.add_space(8.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.key_input)
                    .password(true)
                    .desired_width(f32::INFINITY)
                    .hint_text("Chave da API"),
            );
            ui.add_space(6.0);
            ui.small(concat!(
                "A chave fica somente na memória enquanto o WordCraft estiver aberto. ",
                "Ela não é salva nas preferências nem enviada ao GitHub."
            ));
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Usar nesta sessão").clicked() {
                    if self.key_input.trim().is_empty() {
                        self.status = "Digite uma chave da API para conectar o assistente.".into();
                    } else {
                        self.api_key = Some(Zeroizing::new(self.key_input.trim().to_string()));
                        self.key_input.zeroize();
                        self.status =
                            "Chave carregada nesta sessão. Faça uma solicitação para testar a conexão.".into();
                        close = true;
                    }
                }
                if ui.button("Agora não").clicked() {
                    self.key_input.zeroize();
                    close = true;
                }
                ui.hyperlink_to("Obter chave", "https://platform.openai.com/api-keys");
            });
        });
        if close {
            self.key_dialog_open = false;
        }
    }

    fn show_chat(&mut self, ui: &mut egui::Ui, app: &mut WordApp) {
        ui.horizontal(|ui| {
            ui.label(format!("Modelo: {MODEL}"));
            if ui.button("Configurar chave").clicked() {
                self.key_input.zeroize();
                self.key_dialog_open = true;
            }
        });
        ui.label("Peça para redigir ou organizar conteúdo. O resultado pode ser inserido no documento aberto.");
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::multiline(&mut self.prompt)
                .desired_rows(4)
                .hint_text(
                    "Ex.: Crie uma prova de matemática para o 1º ano, com 10 questões sobre porcentagem e gabarito.",
                ),
        );
        ui.horizontal(|ui| {
            let disabled = self.pending.is_some() || self.api_key.is_none() || self.prompt.trim().is_empty();
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
            egui::ScrollArea::vertical().max_height(210.0).show(ui, |ui| {
                ui.label(text);
            });
            insert_preview = ui.button("Inserir prévia no documento").clicked();
        }
        if insert_preview
            && let Some(text) = self.generated.clone()
        {
            self.insert_into_document(app, text);
        }
    }

    fn start_request(&mut self, insert_when_ready: bool, ctx: &egui::Context) {
        let Some(key) = self.api_key.as_ref() else {
            self.status = "Configure a chave da API antes de enviar uma solicitação.".into();
            self.key_dialog_open = true;
            return;
        };
        let prompt = self.prompt.trim();
        if prompt.is_empty() {
            self.status = "Digite o que deseja criar.".into();
            return;
        }
        if prompt.chars().count() > MAX_PROMPT_CHARS {
            self.status = format!("A solicitação deve ter até {MAX_PROMPT_CHARS} caracteres.");
            return;
        }
        let prompt = prompt.to_string();
        let key = Zeroizing::new(key.as_str().to_string());
        let (tx, rx) = channel();
        let repaint = ctx.clone();
        std::thread::spawn(move || {
            let result = request_openai(&key, &prompt);
            let _ = tx.send(result);
            repaint.request_repaint();
        });
        self.pending = Some(rx);
        self.insert_when_ready = insert_when_ready;
        self.generated = None;
        self.status = "Enviando a solicitação para a API da OpenAI…".into();
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
            Err(error) => {
                self.status = error;
            }
        }
        self.insert_when_ready = false;
    }

    fn insert_into_document(&mut self, app: &mut WordApp, text: String) {
        if text.trim().is_empty() {
            self.status = "A resposta está vazia e não foi inserida.".into();
            return;
        }
        if !app.has_document()
            && let Err(error) = app.run("file.new", json!({"template": "blank"}))
        {
            self.status = format!("Não foi possível criar o documento: {error}");
            return;
        }
        match app.run("text.insert", json!({"text": text, "paragraphs": true})) {
            Ok(_) => self.status = "Conteúdo inserido no documento. Use Desfazer se quiser removê-lo.".into(),
            Err(error) => self.status = format!("Não foi possível inserir o conteúdo: {error}"),
        }
    }
}

fn request_openai(api_key: &str, prompt: &str) -> Result<String, String> {
    let config = ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(90))).build();
    let agent: ureq::Agent = config.into();
    let request = agent
        .post("https://api.openai.com/v1/responses")
        .header("Authorization", &format!("Bearer {api_key}"));
    let body = json!({
        "model": MODEL,
        "instructions": INSTRUCTIONS,
        "input": prompt,
        "max_output_tokens": 4000
    });
    let mut response = request
        .send_json(body)
        .map_err(|_| {
            "A chamada à API falhou. Confira a chave, a conexão e os limites da sua conta OpenAI.".to_string()
        })?;
    let value: Value = response
        .body_mut()
        .read_json()
        .map_err(|_| "A API retornou uma resposta que não pôde ser lida.".to_string())?;
    let mut text = String::new();
    if let Some(output) = value.get("output").and_then(Value::as_array) {
        for item in output {
            if let Some(content) = item.get("content").and_then(Value::as_array) {
                for part in content {
                    if part.get("type").and_then(Value::as_str) == Some("output_text")
                        && let Some(value) = part.get("text").and_then(Value::as_str)
                    {
                        if !text.is_empty() {
                            text.push('\n');
                        }
                        text.push_str(value);
                        if text.chars().count() > MAX_OUTPUT_CHARS {
                            return Err("A resposta passou do limite de tamanho do WordCraft.".into());
                        }
                    }
                }
            }
        }
    }
    if text.trim().is_empty() {
        return Err("A API não retornou texto para inserir.".into());
    }
    Ok(text)
}
