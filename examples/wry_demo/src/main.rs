use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_winit_gtk::winit_runner;
use wry::application::dpi::{PhysicalSize, Size};
use wry::application::event_loop::EventLoop;
use wry::application::window::{Fullscreen, Window as TaoWindow, WindowBuilder};
use wry::webview::{WebView, WebViewBuilder};

mod bridge;
mod links;
// mod print_hierarchy;
// mod webview;

pub struct WryWebview(WebView);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(bevy::log::LogPlugin {
                    level: bevy::log::Level::INFO,
                    filter: "wgpu_core=warn,wgpu_hal=warn".to_string(),
                })
                .set(bevy::window::WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(1280., 720.),
                        ..default()
                    }),
                    ..default()
                }),
            bevy_winit_gtk::WinitPlugin,
            links::LinksPlugin,
        ))
        .add_event::<bridge::Event>()
        .add_systems(Last, (sync_window, bridge::bevy_read_requests_system))
        .add_systems(PostUpdate, bridge::bevy_emit_events_system)
        .set_runner(|mut app| {
            setup_webview(&mut app);
            winit_runner(app);
        })
        .run();
}
fn setup_webview(app: &mut App) {
    let (bevy_receiver, wry_sender) = bridge::make_bridge();
    let event_loop = app.world.non_send_resource::<EventLoop<()>>();

    let webview_window = WindowBuilder::new()
        .with_transparent(true)
        .with_inner_size(PhysicalSize::new(1280., 720.))
        .with_visible(false)
        // .with_always_on_top(true)
        .build(&event_loop)
        .unwrap();
    let webview = WebViewBuilder::new(webview_window)
        .unwrap()
        .with_initialization_script(
            r#"setTimeout(() => {
                    var links = Array.from(document.links);
                    var to_hide = document.querySelectorAll("main, html, .layout, body");
                    to_hide.forEach((item) => item.style = "background: transparent");
                    window.ipc.postMessage(`NavigatedTo:${links.join(',')}`);
                }, 2);
                "#,
        )
        .with_url("https://bevyengine.org")
        .unwrap()
        .with_ipc_handler(move |w, s| bridge::wry_bridge(&wry_sender, w, s))
        .with_transparent(true)
        .build()
        .unwrap();

    app.insert_non_send_resource(WryWebview(webview))
        .insert_non_send_resource(bevy_receiver);
}

fn sync_window(
    main_window: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    webview: NonSend<WryWebview>,
) {
    let Ok(_window) = main_window.get_single() else {
        return;
    };
    let wv_window = webview.0.window();
}
