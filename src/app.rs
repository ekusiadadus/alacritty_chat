use egui::{CentralPanel, SidePanel};
use eframe::App;

use crate::terminal::TerminalPane;
use crate::chat::ChatPanel;

pub struct AppState {
    terminal: TerminalPane,
    chat: ChatPanel,
}

impl AppState {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // カスタムフォントを設定したい場合はここで行う
        let ctx = &cc.egui_ctx;
        
        // ターミナルとチャットパネルの初期化
        let terminal = TerminalPane::new();
        let chat = ChatPanel::new();
        
        Self {
            terminal,
            chat,
        }
    }
}

impl App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 左側にチャットパネル、右側にターミナルを配置
        SidePanel::left("chat_panel")
            .resizable(true)
            .min_width(300.0)
            .show(ctx, |ui| {
                self.chat.ui(ui);
            });
            
        CentralPanel::default().show(ctx, |ui| {
            self.terminal.ui(ui);
        });
    }
} 