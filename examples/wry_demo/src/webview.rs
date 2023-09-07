use bevy::prelude::*;
use bevy_tao::TaoWindows;

fn primary_window_webview(tao_windows: NonSend<TaoWindows>) {
    if !tao_windows.is_changed() {
        return;
    }
}
