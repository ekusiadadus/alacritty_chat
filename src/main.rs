mod app;
mod terminal;
mod chat;

fn main() {
    // X11またはWayland環境のチェック
    let wsl_mode = std::env::var("WSL_DISTRO_NAME").is_ok();
    
    // ディスプレイ環境の確認
    let has_display = std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
    
    // WSL環境で表示サーバーが設定されていない場合、ヘルプを表示
    if wsl_mode && !has_display {
        println!("WSL2環境で実行するには、X Serverの設定が必要です。");
        println!("以下の手順で設定してください：");
        println!("1. Windows側でVcXsrvなどのX Serverを起動");
        println!("2. WSL2側で次のコマンドを実行: export DISPLAY=$(cat /etc/resolv.conf | grep nameserver | awk '{{print $2}}'):0");
        println!("3. 再度アプリケーションを起動");
        return;
    }
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    
    match eframe::run_native(
        "Alacritty Chat",
        options,
        Box::new(|cc| Ok(Box::new(app::AppState::new(cc))))
    ) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("アプリケーションの起動に失敗しました: {}", e);
            
            if wsl_mode {
                println!("\nWSL2環境では、以下の設定を確認してください：");
                println!("1. Windows側でX Serverが実行中か");
                println!("2. WSL2側で環境変数DISPLAYが正しく設定されているか");
                println!("3. Firewallで接続がブロックされていないか");
                
                // 適切なDISPLAY設定値の提案
                if let Ok(output) = std::process::Command::new("sh")
                    .arg("-c")
                    .arg("cat /etc/resolv.conf | grep nameserver | awk '{print $2}'")
                    .output() 
                {
                    if let Ok(ip) = String::from_utf8(output.stdout) {
                        println!("\n推奨設定: export DISPLAY={}:0.0", ip.trim());
                    }
                }
            }
        }
    }
}
