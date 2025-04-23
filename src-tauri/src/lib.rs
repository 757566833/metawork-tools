use chrono::{Local, Timelike};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, PhysicalPosition, PhysicalSize, Window, WindowEvent,
};
use tauri_plugin_dialog::DialogExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn start_polling(app_handle: &mut tauri::App, window: tauri::WebviewWindow) {
    app_handle
        .dialog()
        .message("尽快登录进入生产线首页以便工具检测，8点前16点后不检测")
        .title("每10分钟检测一次虚拟产线有没有开启")
        .show(|_result| {
           
        });
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(600)); // 每 600 秒执行一次

            // JS 代码：获取 class 内容并调用 Rust 接口回传
            let js_code = r#"
                (function() {
                    const el = document.querySelector('.task-bar.work');
                    const content = el ? el.textContent : null;

                    if (!content) {
                        window.__TAURI__.core.invoke('not_work');
                    }
                })();
            "#;

            // let js_code = r#"
            //     console.log("hello word from rust")
            // "#;

            if let Err(err) = window.eval(js_code) {
                eprintln!("JS eval error: {:?}", err);
            }
        }
    });
}

// const LOCKED: OnceCell<bool> = OnceCell::new();
#[tauri::command]
fn not_work(window: Window) {
    let current_time = Local::now();
    // let binding = LOCKED;
    // let locked = binding.get().unwrap_or(&false);
    // if *locked {
    //     return;
    // }
    if current_time.hour() < 16 && current_time.hour() > 8 {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
        let _ = window.set_size(PhysicalSize::new(800.0, 240.0));

        // 设置窗口位置到屏幕的左上角 (0, 0)
        let _ = window.set_position(PhysicalPosition::new(0.0, 0.0));
        // 可存入文件、数据库、上报远程等
        // dialog::message("警告", "这是一个警告对话框");

        // let set_result = LOCKED.set(true);
        // if set_result.is_err() {
        //     // 处理错误
        //     println!("Failed to set the value 1");
        // }

        // let result = binding.get();


        // println!("not_work {}", binding.get().unwrap());

        // app_handle
        //     .dialog()
        //     .message("Tauri is Awesome")
        //     .title("Tauri is Awesome")
        //     .show(|_result| {
        //         let set_result = LOCKED.set(false);
        //         if set_result.is_err() {
        //             // 处理错误
        //             println!("Failed to set the value 2");
        //         }
        //     });
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    // let menu = Menu::with_items(app, &[&quit_i])?;
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app
                .get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;
            let _ = TrayIconBuilder::new()
                .menu(&menu)
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        println!("left click pressed and released");
                        // in this example, let's show and focus the main window when the tray is clicked
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {
                        // println!("unhandled event {event:?}");
                    }
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        // println!("quit menu item was clicked");
                        app.exit(0);
                    }
                    _ => {
                        // println!("menu item {:?} not handled", event.id);
                    }
                })
                .icon(app.default_window_icon().unwrap().clone())
                .build(app);
            let window = app.get_webview_window("main").unwrap(); // 获取主窗口
            start_polling(app, window); // 启动定时任务
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                WindowEvent::CloseRequested { api, .. } => {
                    // 阻止关闭窗口
                    api.prevent_close();
                    // 隐藏窗口（进入托盘）
                    let _ = window.hide();
                }
                WindowEvent::Resized(physical_size) => {
                    if physical_size.width == 0 && physical_size.height == 0 {
                        // 隐藏窗口（进入托盘）
                        let _ = window.hide();
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![greet, not_work])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
