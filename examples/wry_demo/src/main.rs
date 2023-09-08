use bevy::prelude::*;
use bevy_tao::GetWindow;
use wry::webview::{WebView, WebViewBuilder};

mod bridge;
mod links;
// mod print_hierarchy;
// mod webview;

pub struct WryWebview(WebView);

impl GetWindow for WryWebview {
    fn get_window(&self) -> &bevy_tao::TaoWindow {
        self.0.window()
    }
    fn wrap(window: bevy_tao::TaoWindow) -> Self {
        info!("Wrapping a window");
        let webview = WebViewBuilder::new(window)
            .unwrap()
            .with_initialization_script(
                r#"setTimeout(() => {
                    var links = Array.from(document.links);
                    document.querySelector("main").style = "background: transparent!important";
                    document.querySelector(".layout").style = "background: transparent!important";
                    window.ipc.postMessage(`NavigatedTo:${links.join(',')}`);
                }, 400);
                "#,
            )
            .with_url("https://bevyengine.org")
            .unwrap()
            .with_ipc_handler(bridge::wry_bridge)
            .with_transparent(true)
            .with_visible(false)
            .build()
            .unwrap();
        WryWebview(webview)
    }
}

fn main() {
    let (bevy_receiver, wry_sender) = bridge::make_bridge();
    bridge::WRY_SENDER.set(wry_sender).unwrap();

    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                level: bevy::log::Level::INFO,
                filter: "wgpu_core=warn,wgpu_hal=warn".to_string(),
            }),
            bevy_tao::TaoPlugin::<WryWebview>::default(),
            links::LinksPlugin,
        ))
        .insert_resource(ClearColor(Color::rgba(0., 0., 0., 0.)))
        .insert_non_send_resource(bevy_receiver)
        .add_event::<bridge::Event>()
        .add_systems(Last, bridge::bevy_read_requests_system)
        .add_systems(PostUpdate, bridge::bevy_emit_events_system)
        .run();
}
