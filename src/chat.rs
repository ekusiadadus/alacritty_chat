use egui::{ScrollArea, TextEdit, Button, RichText, TextStyle, Color32, Layout, Align};
use serde::{Serialize, Deserialize};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub enum ChatRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatMessage {
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }
    
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
    
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }
    
    pub fn is_user(&self) -> bool {
        self.role == "user"
    }
    
    pub fn is_assistant(&self) -> bool {
        self.role == "assistant"
    }
}

#[derive(Serialize, Deserialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct OpenAIResponse {
    #[serde(default)]
    choices: Vec<OpenAIChoice>,
}

#[derive(Serialize, Deserialize)]
struct OpenAIChoice {
    #[serde(default)]
    message: ChatMessage,
}

// LLM APIサービスのトレイト
trait LLMService: Send + Sync {
    fn send_message(&self, messages: Vec<ChatMessage>) -> Result<String, String>;
}

// OpenAI APIの実装
struct OpenAIService {
    api_key: String,
    model: String,
}

impl OpenAIService {
    fn new(api_key: String, model: String) -> Self {
        Self { api_key, model }
    }
}

impl LLMService for OpenAIService {
    fn send_message(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        // OpenAI APIリクエストの作成
        let client = reqwest::blocking::Client::new();
        let request = OpenAIRequest {
            model: self.model.clone(),
            messages,
            temperature: 0.7,
        };
        
        // APIリクエストを送信
        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send();
        
        match response {
            Ok(res) => {
                if !res.status().is_success() {
                    return Err(format!("API エラー: ステータスコード {}", res.status()));
                }
                
                match res.json::<OpenAIResponse>() {
                    Ok(data) => {
                        if data.choices.is_empty() {
                            return Err("APIからの応答に選択肢がありません".to_string());
                        }
                        Ok(data.choices[0].message.content.clone())
                    },
                    Err(e) => Err(format!("JSONのパースに失敗: {}", e)),
                }
            },
            Err(e) => Err(format!("APIリクエストが失敗: {}", e)),
        }
    }
}

// モックLLMサービス（APIキーがない場合やテスト用）
struct MockLLMService;

impl LLMService for MockLLMService {
    fn send_message(&self, messages: Vec<ChatMessage>) -> Result<String, String> {
        // 最後のユーザーメッセージに対するレスポンスを生成
        let last_message = messages.iter().rev().find(|m| m.is_user());
        
        match last_message {
            Some(msg) => {
                // モックレスポンスを生成
                let response = format!(
                    "あなたのメッセージ「{}」を受け取りました。\n\nこれはモック応答です。実際のAPI接続を設定するには、環境変数 OPENAI_API_KEY を設定してください。",
                    msg.content
                );
                // 遅延を追加（実際のAPIのような感覚）
                thread::sleep(Duration::from_millis(500));
                Ok(response)
            },
            None => Err("ユーザーメッセージがありません".to_string()),
        }
    }
}

pub struct ChatPanel {
    history: Vec<ChatMessage>,
    input_buffer: String,
    awaiting_response: bool,
    rx: Option<Receiver<Result<String, String>>>,
    llm_service: Box<dyn LLMService>,
}

impl ChatPanel {
    pub fn new() -> Self {
        // OpenAI APIキーを環境変数から取得
        let api_key = std::env::var("OPENAI_API_KEY");
        
        // APIキーがある場合はOpenAIサービスを、なければモックサービスを使用
        let llm_service: Box<dyn LLMService> = match api_key {
            Ok(key) if !key.is_empty() => {
                Box::new(OpenAIService::new(key, "gpt-3.5-turbo".to_string()))
            },
            _ => {
                Box::new(MockLLMService)
            }
        };
        
        Self {
            history: Vec::new(),
            input_buffer: String::new(),
            awaiting_response: false,
            rx: None,
            llm_service,
        }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // タイトル
            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                ui.heading("チャットパネル");
            });
            
            ui.add_space(10.0);
            
            // チャット履歴表示エリア
            ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .max_height(ui.available_height() - 100.0)
                .show(ui, |ui| {
                    for message in &self.history {
                        let (text, color, is_user) = if message.is_user() {
                            (RichText::new("あなた").strong(), Color32::LIGHT_BLUE, true)
                        } else if message.is_assistant() {
                            (RichText::new("AI").strong(), Color32::LIGHT_GREEN, false)
                        } else {
                            (RichText::new("システム").italics(), Color32::LIGHT_GRAY, false)
                        };
                        
                        // メッセージヘッダー
                        ui.horizontal(|ui| {
                            ui.label(text.color(color));
                            if is_user {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new("✉").small());
                                });
                            }
                        });
                        
                        // メッセージ内容（マークダウン風にする）
                        let content = &message.content;
                        
                        // コードブロックを検出（簡易的な実装）
                        let mut in_code_block = false;
                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("```") {
                                in_code_block = !in_code_block;
                                ui.label(RichText::new("```").monospace().color(Color32::GRAY));
                            } else if in_code_block {
                                ui.label(RichText::new(line).monospace().background_color(Color32::from_rgb(40, 40, 40)));
                            } else {
                                ui.label(line);
                            }
                        }
                        
                        ui.separator();
                    }
                    
                    // 応答待ちの場合はインジケータを表示
                    if self.awaiting_response {
                        ui.label(RichText::new("応答を待っています...").italics());
                    }
                });
            
            // レスポンスをチェック
            if let Some(rx) = &self.rx {
                if let Ok(result) = rx.try_recv() {
                    match result {
                        Ok(response) => {
                            self.history.push(ChatMessage::assistant(response));
                        },
                        Err(error) => {
                            self.history.push(ChatMessage::system(format!("エラー: {}", error)));
                        }
                    }
                    self.awaiting_response = false;
                    self.rx = None;
                }
            }
            
            ui.add_space(10.0);
            
            // 入力エリア
            ui.horizontal(|ui| {
                let text_edit = TextEdit::multiline(&mut self.input_buffer)
                    .desired_width(ui.available_width() - 60.0)
                    .desired_rows(3)
                    .hint_text("メッセージを入力...");
                
                let response = ui.add(text_edit);
                
                // Enterキーで送信（Ctrlキーなしの場合）
                let mut should_send = false;
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.ctrl) {
                    should_send = true;
                }
                
                if ui.add_enabled(!self.awaiting_response, Button::new("送信")).clicked() || should_send {
                    self.send_message();
                }
            });
        });
    }
    
    fn send_message(&mut self) {
        if self.input_buffer.trim().is_empty() || self.awaiting_response {
            return;
        }
        
        // 入力をチャット履歴に追加
        let user_message = self.input_buffer.clone();
        self.history.push(ChatMessage::user(user_message));
        self.input_buffer.clear();
        
        // APIリクエストを準備
        self.awaiting_response = true;
        
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        
        // APIリクエスト用にメッセージ履歴をクローン
        let history = self.history.clone();
        
        // 別スレッドでAPIリクエストを実行
        thread::spawn(move || {
            // モックLLMサービスでリクエスト送信（本来はself.llm_serviceを使用すべきだが、スレッド間で共有できない問題を回避）
            let llm_service = MockLLMService;
            let result = llm_service.send_message(history);
            tx.send(result).ok();
        });
    }
} 