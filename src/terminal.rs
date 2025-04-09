use egui::{Color32, Align2, Rect, Vec2, Pos2};
use std::{
    sync::mpsc,
    thread,
    time::Duration,
    process::{Command, Stdio},
    io::{Read, Write},
    env,
};

// ターミナルの色設定
const COLORS: [Color32; 16] = [
    Color32::from_rgb(0, 0, 0),       // 黒 (背景色)
    Color32::from_rgb(205, 0, 0),     // 赤
    Color32::from_rgb(0, 205, 0),     // 緑
    Color32::from_rgb(205, 205, 0),   // 黄
    Color32::from_rgb(0, 0, 238),     // 青
    Color32::from_rgb(205, 0, 205),   // マゼンタ
    Color32::from_rgb(0, 205, 205),   // シアン
    Color32::from_rgb(229, 229, 229), // 白
    Color32::from_rgb(127, 127, 127), // 明るい黒
    Color32::from_rgb(255, 0, 0),     // 明るい赤
    Color32::from_rgb(0, 255, 0),     // 明るい緑
    Color32::from_rgb(255, 255, 0),   // 明るい黄
    Color32::from_rgb(92, 92, 255),   // 明るい青
    Color32::from_rgb(255, 0, 255),   // 明るいマゼンタ
    Color32::from_rgb(0, 255, 255),   // 明るいシアン
    Color32::from_rgb(255, 255, 255), // 明るい白
];

enum TerminalEvent {
    Input(Vec<u8>),
}

pub struct TerminalPane {
    event_tx: mpsc::Sender<TerminalEvent>,
    output_rx: mpsc::Receiver<String>,
    buffer: Vec<String>,
    cursor_pos: (usize, usize), // (column, row)
    size: (u16, u16), // (cols, rows)
    cell_size: (f32, f32), // (width, height)in pixels
    focused: bool,
}

impl TerminalPane {
    pub fn new() -> Self {
        // 端末の初期サイズ（列数と行数）
        let cols = 80;
        let rows = 24;
        
        // フォントサイズに基づいたセルのサイズ
        let cell_width = 8.0;  // 仮の値、実際にはフォントメトリクスから計算
        let cell_height = 16.0; // 仮の値
        
        // イベント送信用のチャネル
        let (event_tx, event_rx) = mpsc::channel();
        let (output_tx, output_rx) = mpsc::channel();
        
        // 環境変数からシェルを取得
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "cmd.exe".to_string()
            } else {
                "/bin/bash".to_string()
            }
        });
        
        // ターミナルスレッドを起動
        thread::spawn(move || {
            // 環境変数を設定
            let mut command = Command::new(&shell);
            command.stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            
            // 環境変数を設定して日本語対応
            let path = env::var("PATH").unwrap_or_default();
            let home = env::var("HOME").unwrap_or_default();
            
            command.env("PATH", path)
                   .env("HOME", home)
                   .env("LANG", "ja_JP.UTF-8")
                   .env("LC_ALL", "ja_JP.UTF-8")
                   .env("TERM", "xterm-256color");
            
            // シェルプロセスを起動
            let mut child = match command.spawn() {
                Ok(child) => {
                    // 成功したら情報をログに出力
                    output_tx.send(format!("ターミナルを起動しました: {}\n", shell)).ok();
                    child
                },
                Err(e) => {
                    // 詳細なエラー情報を送信
                    let error_msg = format!("シェルの起動に失敗しました: {}\nパス: {}\n", e, shell);
                    output_tx.send(error_msg).ok();
                    
                    // フォールバック: もしbashが失敗したら他のシェルを試す
                    if shell == "/bin/bash" {
                        // shを試す
                        match Command::new("/bin/sh")
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn() {
                                Ok(child) => {
                                    output_tx.send("フォールバック: /bin/shを使用します\n".to_string()).ok();
                                    child
                                },
                                Err(e2) => {
                                    output_tx.send(format!("すべてのシェルが失敗しました: {}\n", e2)).ok();
                                    return;
                                }
                            }
                    } else {
                        return;
                    }
                }
            };
            
            // 標準入力へのハンドルを取得
            let mut stdin = match child.stdin.take() {
                Some(stdin) => stdin,
                None => {
                    output_tx.send("子プロセスのstdinを取得できません\n".to_string()).ok();
                    return;
                }
            };
            
            // 標準出力を読み取るスレッドを起動
            let mut stdout = match child.stdout.take() {
                Some(stdout) => stdout,
                None => {
                    output_tx.send("子プロセスのstdoutを取得できません\n".to_string()).ok();
                    return;
                }
            };
            
            let output_tx_clone = output_tx.clone();
            
            thread::spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    match stdout.read(&mut buffer) {
                        Ok(0) => break, // EOFに達した
                        Ok(n) => {
                            match String::from_utf8(buffer[..n].to_vec()) {
                                Ok(s) => {
                                    output_tx_clone.send(s).ok();
                                },
                                Err(_) => {
                                    // UTF-8でデコードできない場合は、lossy変換を使用
                                    let s = String::from_utf8_lossy(&buffer[..n]).to_string();
                                    output_tx_clone.send(s).ok();
                                }
                            }
                        },
                        Err(e) => {
                            output_tx_clone.send(format!("stdout読み取りエラー: {}\n", e)).ok();
                            break;
                        }
                    }
                }
            });
            
            // 標準エラー出力を読み取るスレッド
            let mut stderr = match child.stderr.take() {
                Some(stderr) => stderr,
                None => {
                    output_tx.send("子プロセスのstderrを取得できません\n".to_string()).ok();
                    return;
                }
            };
            
            thread::spawn(move || {
                let mut buffer = [0; 1024];
                loop {
                    match stderr.read(&mut buffer) {
                        Ok(0) => break, // EOFに達した
                        Ok(n) => {
                            match String::from_utf8(buffer[..n].to_vec()) {
                                Ok(s) => {
                                    output_tx.send(s).ok();
                                },
                                Err(_) => {
                                    // UTF-8でデコードできない場合は、lossy変換を使用
                                    let s = String::from_utf8_lossy(&buffer[..n]).to_string();
                                    output_tx.send(s).ok();
                                }
                            }
                        },
                        Err(e) => {
                            output_tx.send(format!("stderr読み取りエラー: {}\n", e)).ok();
                            break;
                        }
                    }
                }
            });
            
            // キー入力を処理するループ
            for event in event_rx {
                match event {
                    TerminalEvent::Input(data) => {
                        if stdin.write_all(&data).is_err() {
                            break;
                        }
                        if stdin.flush().is_err() {
                            break;
                        }
                    }
                }
            }
        });
        
        Self {
            event_tx,
            output_rx,
            buffer: vec![String::new(); rows as usize],
            cursor_pos: (0, 0),
            size: (cols, rows),
            cell_size: (cell_width, cell_height),
            focused: false,
        }
    }
    
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // ターミナル領域をインタラクティブな領域として設定
        let response = ui.allocate_response(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );
        
        if response.clicked() {
            self.focused = true;
            // フォーカスを設定
            ui.memory_mut(|mem| mem.request_focus(response.id));
        }
        
        let painter = ui.painter();
        let rect = response.rect;
        
        // 現在のレンダリング領域のサイズから端末のサイズを再計算
        let new_cols = (rect.width() / self.cell_size.0).floor() as u16;
        let new_rows = (rect.height() / self.cell_size.1).floor() as u16;
        
        // 端末サイズが変更された場合はバッファサイズを調整
        if new_cols != self.size.0 || new_rows != self.size.1 {
            if new_cols > 0 && new_rows > 0 {
                self.size = (new_cols, new_rows);
                
                // バッファサイズを調整
                if self.buffer.len() < new_rows as usize {
                    self.buffer.resize(new_rows as usize, String::new());
                }
            }
        }
        
        // 出力を読み取り、バッファに追加
        self.process_output();
        
        // 背景を描画
        painter.rect_filled(rect, 0.0, COLORS[0]);
        
        // 各行を描画
        for (row, line) in self.buffer.iter().enumerate().take(self.size.1 as usize) {
            let y = rect.min.y + row as f32 * self.cell_size.1;
            
            // 行の内容を描画
            if !line.is_empty() {
                painter.text(
                    Pos2::new(rect.min.x, y),
                    Align2::LEFT_TOP,
                    line,
                    egui::FontId::monospace(self.cell_size.1 * 0.8),
                    COLORS[7] // 白
                );
            }
        }
        
        // カーソルを描画
        if self.cursor_pos.1 < self.size.1 as usize {
            let cursor_x = rect.min.x + self.cursor_pos.0 as f32 * self.cell_size.0;
            let cursor_y = rect.min.y + self.cursor_pos.1 as f32 * self.cell_size.1;
            
            let cursor_rect = Rect::from_min_size(
                Pos2::new(cursor_x, cursor_y),
                Vec2::new(self.cell_size.0, self.cell_size.1)
            );
            
            painter.rect_filled(cursor_rect, 0.0, Color32::from_rgba_unmultiplied(200, 200, 200, 128));
        }
        
        // フォーカスがある場合はキーボード入力を処理
        if self.focused && response.has_focus() {
            let input = ui.input(|i| {
                // 入力イベントを処理
                let mut input_bytes = Vec::new();
                
                // キー入力を処理
                for event in &i.events {
                    if let egui::Event::Key {
                        key, pressed: true, modifiers, ..
                    } = event
                    {
                        // Ctrl修飾キーの処理
                        if modifiers.ctrl {
                            // Ctrlキーと組み合わせた文字キーの処理
                            if let Some(c) = key.symbol_or_name().chars().next() {
                                if c.is_ascii_alphabetic() {
                                    // Ctrl+A～Zは1～26のコードに変換
                                    let code = (c.to_ascii_uppercase() as u8 - b'A' + 1) as char;
                                    input_bytes.push(code as u8);
                                    continue;
                                }
                            }
                        }
                        
                        // 基本的なキーマッピング
                        let bytes = match key {
                            egui::Key::Enter => b"\r".to_vec(),
                            egui::Key::Escape => b"\x1b".to_vec(),
                            egui::Key::Tab => b"\t".to_vec(),
                            egui::Key::Backspace => b"\x7f".to_vec(),
                            egui::Key::Delete => b"\x1b[3~".to_vec(),
                            egui::Key::ArrowUp => b"\x1b[A".to_vec(),
                            egui::Key::ArrowDown => b"\x1b[B".to_vec(),
                            egui::Key::ArrowRight => b"\x1b[C".to_vec(),
                            egui::Key::ArrowLeft => b"\x1b[D".to_vec(),
                            egui::Key::Home => b"\x1b[H".to_vec(),
                            egui::Key::End => b"\x1b[F".to_vec(),
                            egui::Key::PageUp => b"\x1b[5~".to_vec(),
                            egui::Key::PageDown => b"\x1b[6~".to_vec(),
                            _ => Vec::new(),
                        };
                        
                        if !bytes.is_empty() {
                            input_bytes.extend_from_slice(&bytes);
                        }
                    }
                }
                
                // 通常の文字入力
                for c in i.events.iter().filter_map(|e| {
                    if let egui::Event::Text(text) = e {
                        Some(text.as_str())
                    } else {
                        None
                    }
                }) {
                    input_bytes.extend_from_slice(c.as_bytes());
                }
                
                input_bytes
            });
            
            // 入力があれば送信
            if !input.is_empty() {
                let _ = self.event_tx.send(TerminalEvent::Input(input));
            }
        }
    }
    
    // 出力を処理してバッファに追加
    fn process_output(&mut self) {
        // 非ブロッキングで出力を読み取る
        while let Ok(output) = self.output_rx.try_recv() {
            // 制御シーケンスの基本的な処理（完全なANSIパーサーではありません）
            for c in output.chars() {
                match c {
                    '\r' => {
                        // キャリッジリターン: カーソルを行の先頭に移動
                        self.cursor_pos.0 = 0;
                    },
                    '\n' => {
                        // 改行: カーソルを次の行に移動
                        self.cursor_pos.1 += 1;
                        if self.cursor_pos.1 >= self.buffer.len() {
                            // バッファの最後に達したら、新しい行を追加
                            self.buffer.push(String::new());
                            
                            // バッファサイズが大きくなりすぎた場合は、古い行を削除
                            if self.buffer.len() > self.size.1 as usize * 3 {
                                let excess = self.buffer.len() - self.size.1 as usize * 2;
                                self.buffer.drain(0..excess);
                                self.cursor_pos.1 -= excess;
                            }
                        }
                    },
                    '\t' => {
                        // タブ: 8文字分のスペース
                        for _ in 0..8 {
                            if self.cursor_pos.0 < self.size.0 as usize {
                                if self.cursor_pos.1 >= self.buffer.len() {
                                    self.buffer.push(String::new());
                                }
                                while self.buffer[self.cursor_pos.1].len() <= self.cursor_pos.0 {
                                    self.buffer[self.cursor_pos.1].push(' ');
                                }
                                self.cursor_pos.0 += 1;
                            }
                        }
                    },
                    '\x08' => {
                        // バックスペース: カーソルを1つ左に移動
                        if self.cursor_pos.0 > 0 {
                            self.cursor_pos.0 -= 1;
                        }
                    },
                    _ => {
                        // 通常の文字: バッファに追加
                        if !c.is_control() {
                            if self.cursor_pos.1 >= self.buffer.len() {
                                self.buffer.push(String::new());
                            }
                            
                            // 現在の行が短い場合は、カーソル位置までスペースで埋める
                            let line = &mut self.buffer[self.cursor_pos.1];
                            while line.len() < self.cursor_pos.0 {
                                line.push(' ');
                            }
                            
                            // 文字を追加（安全に行う）
                            if self.cursor_pos.0 >= line.len() {
                                // 新しい文字を追加
                                line.push(c);
                            } else {
                                // 既存の文字列を分割して挿入
                                let mut new_line = String::new();
                                for (i, ch) in line.chars().enumerate() {
                                    if i == self.cursor_pos.0 {
                                        new_line.push(c);
                                    }
                                    new_line.push(ch);
                                }
                                *line = new_line;
                            }
                            
                            self.cursor_pos.0 += 1;
                            if self.cursor_pos.0 >= self.size.0 as usize {
                                self.cursor_pos.0 = 0;
                                self.cursor_pos.1 += 1;
                                if self.cursor_pos.1 >= self.buffer.len() {
                                    self.buffer.push(String::new());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    // キーボード入力を処理するメソッド
    pub fn handle_key_press(&mut self, input: &str) {
        let _ = self.event_tx.send(TerminalEvent::Input(input.as_bytes().to_vec()));
    }
} 