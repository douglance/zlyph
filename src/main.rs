mod actions;
mod editor;
mod theme;

use actions::*;
use editor::TextEditor;
use gpui::*;

fn main() {
    Application::new().run(|app| {
        app.bind_keys([
            KeyBinding::new("cmd-=", IncreaseFontSize, None),
            KeyBinding::new("cmd--", DecreaseFontSize, None),
            KeyBinding::new("cmd-0", ResetFontSize, None),
            KeyBinding::new("cmd-a", SelectAll, None),
            KeyBinding::new("cmd-c", Copy, None),
            KeyBinding::new("cmd-x", Cut, None),
            KeyBinding::new("cmd-v", Paste, None),
            KeyBinding::new("enter", Newline, None),
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("delete", Delete, None),
            KeyBinding::new("cmd-backspace", DeleteToBeginningOfLine, None),
            KeyBinding::new("cmd-delete", DeleteToEndOfLine, None),
            KeyBinding::new("cmd-left", MoveToBeginningOfLine, None),
            KeyBinding::new("cmd-right", MoveToEndOfLine, None),
            KeyBinding::new("left", MoveLeft, None),
            KeyBinding::new("right", MoveRight, None),
            KeyBinding::new("up", MoveUp, None),
            KeyBinding::new("down", MoveDown, None),
            KeyBinding::new("alt-left", MoveWordLeft, None),
            KeyBinding::new("alt-right", MoveWordRight, None),
        ]);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: Point { x: px(100.0), y: px(100.0) },
                size: Size { width: px(800.0), height: px(600.0) },
            })),
            titlebar: Some(TitlebarOptions {
                title: Some("Dright Editor".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(8.0), px(8.0))),
            }),
            window_background: WindowBackgroundAppearance::Blurred,
            ..Default::default()
        };

        app.open_window(window_options, |_window, app| {
            app.new(|cx| TextEditor::new(cx))
        })
        .unwrap();
    });
}
