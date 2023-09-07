use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::OnceLock;

use bevy::{
    ecs::archetype::{Archetype, ArchetypeEntity},
    prelude::*,
};
use bevy_tao::TaoWindows;
use wry::application::{
    event::{Event as WryEvent, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopWindowTarget},
};
use wry::webview::RequestAsyncResponder;

use crate::WryWebview;

static WRY_SENDER: OnceLock<WrySender> = OnceLock::new();

struct WrySender(Sender<Request>);

pub struct BevyChannels {
    from_wry: Receiver<Request>,
    to_wry: Sender<Event>,
}

/// Events sent from bevy to wry.
pub enum Event {
    EntityName(Entity, String),
    EntityCount(u32),
    Entities(Box<[Entity]>),
}
impl Event {
    fn command(&self) -> String {
        let str = 
    match self {
        Event::EntityName(_,_) => {
            "console.log(\"EntityName\")"
        },
        Event::EntityCount(_) => {
            "console.log(\"EntityCount\")"
        },
        Event::Entities(_) => {
            "console.log(\"Entities\")"
        },
    };
        str.to_string()
    }
}
/// Eventy sent to bevy from wry.
pub enum Request {
    PrintHierarchy,
    GetEntities,
    GetEntityName(Entity),
    GetEntityCount,
}

pub fn wry_bridge(request: wry::http::Request<Vec<u8>>, responder: RequestAsyncResponder) {
    let Some(bridge) = WRY_SENDER.get() else {
        eprintln!("wry sender not yet ready");
        return;
    };
    let path = request.uri().path();
}

fn to_wry(world: &mut World, event: Event) {
    let Some(webviews) = world.get_non_send_resource::<TaoWindows<WryWebview>>() else {
        return;
    };
    for webview in webviews.windows.values() {
        webview.0.evaluate_script(&event.command());
    }
}
pub fn bevy_bridge_system(world: &mut World) {
    let channels = world.remove_non_send_resource();
    let Some(BevyChannels { from_wry, to_wry: _ }) = &channels else {
        return;
    };
    for request in from_wry.try_iter() {
        match request {
            Request::PrintHierarchy => {
                let mut q = world.query_filtered::<Entity, Without<Parent>>();
                for entity in q.iter(world) {
                    println!("{entity:?}");
                }
            }
            Request::GetEntities => {
                let archetypes = world.archetypes();
                let entities = archetypes.iter().flat_map(Archetype::entities);
                let entities: Vec<_> = entities.map(ArchetypeEntity::entity).collect();

                to_wry(world, Event::Entities(entities.into()));
            }
            Request::GetEntityName(entity) => {
                let mut q = world.query::<&Name>();
                let Ok(name) = q.get(world, entity) else {
                    continue;
                };
                to_wry(world, Event::EntityName(entity, name.to_string()));
            }
            Request::GetEntityCount => {
                to_wry(world, Event::EntityCount(world.entities().len()));
            }
        }
    }
    world.insert_non_send_resource(channels);
}

pub fn make_bridge() -> (
    BevyChannels,
    impl FnMut(WryEvent<'_, ()>, &EventLoopWindowTarget<()>, &mut ControlFlow) + 'static,
) {
    let (write_from_bevy, read_from_bevy) = channel();
    let (write_from_wry, read_from_wry) = channel();

    let bevy_wry_channel = BevyChannels {
        to_wry: write_from_bevy,
        from_wry: read_from_wry,
    };
    (bevy_wry_channel, move |event, _, control_flow: &mut _| {
        *control_flow = ControlFlow::Wait;

        let to_bevy = &write_from_wry;
        let from_bevy = &read_from_bevy;

        match event {
            WryEvent::NewEvents(StartCause::Init) => println!("Wry has started!"),
            WryEvent::MainEventsCleared => {
                react_to_bevy_events(from_bevy, to_bevy);
            }
            WryEvent::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    })
}

fn react_to_bevy_events(from_bevy: &Receiver<Event>, _to_bevy: &Sender<Request>) {
    for events in from_bevy.try_iter() {
        match events {
            Event::EntityName(_, _) => todo!(),
            Event::EntityCount(_) => todo!(),
            Event::Entities(_) => todo!(),
        }
    }
}
