use std::sync::mpsc::{channel, Receiver, Sender};

use bevy::prelude::*;
use wry::application::window::Window as TaoWindow;

use crate::links::NewPage;
use crate::WryWebview;

/// Eventy sent to bevy from wry.
pub enum Request {
    SpawnNewLinks(Vec<String>),
}

/// Events sent from bevy to wry.
#[derive(Event)]
pub enum Event {
    NavigateToPage(String),
}

#[derive(Deref, Debug)]
pub struct WrySender(Sender<Request>);
pub struct BevyReceiver(Receiver<Request>);

impl Drop for BevyReceiver {
    fn drop(&mut self) {
        error!("Dropping the bevy receiver, If this isn't the last log message, this is very suss");
    }
}

impl Event {
    fn command(&self) -> String {
        match self {
            Event::NavigateToPage(page) => format!("window.location.assign({page:?})"),
        }
    }
    fn to_wry(&self, webview: &WryWebview) {
        webview.0.evaluate_script(&self.command()).unwrap();
    }
}

/// To use in `GetWindow` impl for `WryWebview`.
pub fn wry_bridge(bridge: &WrySender, _window: &TaoWindow, request: String) {
    let prefix = "NavigatedTo:";
    let prefix_len = prefix.len();
    if request.starts_with(prefix) {
        info!("recognized request NavigatedTo");
        let links = &request[prefix_len..];
        let links: Vec<_> = links.split(',').map(str::to_string).collect();
        bridge.send(Request::SpawnNewLinks(links)).unwrap();
    } else {
        error!("unrecognized request: {request}");
    }
}

pub fn bevy_emit_events_system(webview: NonSend<WryWebview>, mut events: EventReader<Event>) {
    for event in events.iter() {
        event.to_wry(&webview);
    }
}

pub fn bevy_read_requests_system(world: &mut World) {
    let receiver_resource: BevyReceiver = world.remove_non_send_resource().unwrap();
    for request in receiver_resource.0.try_iter() {
        match request {
            Request::SpawnNewLinks(links) => {
                info!("Got request: SpawnNewLinks");
                world.send_event(NewPage { links })
            }
        }
    }
    world.insert_non_send_resource(receiver_resource);
}

pub fn make_bridge() -> (BevyReceiver, WrySender) {
    let (wry_sender, bevy_receiver) = channel();
    (BevyReceiver(bevy_receiver), WrySender(wry_sender))
}
