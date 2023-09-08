# Bevy wry demo

Shows 2-ways communication between a webpage running in a webview and bevy.

## What it is

- The webview navigates to `https://bevyengine.org`.
- A javascript snippet reads all the links on the page and spawns a sprite in bevy per link.
- Bevy highlights a different sprite each 3 second, showing the associated link
- (currently not working) when pressing space bar, bevy will send a request to
  the webview to move to the highlighted link
- Whenever navigating to a new page, the sprites will be replaced by new ones
  corresponding to the links on the new page (yay hypertext)

This is mostly a copy of `contributors.rs` from the bevy repository.

## How it works

There are two componts to the current hack enabling this to work:

1. A fork of `bevy_winit` that stores `wry::WebView`s instead of `winit::Window`s.
   It exposes the `GetWindow` trait to let end-user customize what is stored in
   the `windows` resource. `GetWindow` is implemented in `wry_demo` for `wry::WebView`.
   (or rather a newtype, since it would otherwise violate orphan rules).
   The implementation converts the window into a webview and stores the webview.
2. A two-way communcation setup between the Javascript webview runtime and the
   bevy world, defined in `wry_demo::bridge`.

In `bridge` we define two enums. `Request` are events sent from the Javascript
world to the bevy world, while `Event` are events going from bevy to Javascript.

In `bridge` we define the `bevy_read_requests_system`, a bevy system that reads
events from a `Receiver<Request>`, and sends bevy events based on the received
requests.

In `bridge`, we define another system, `bevy_emit_events_system`. It reads
`Event` from a bevy `EventReader<Event>`. Those events are emitted by other
bevy systems. In `bevy_emit_events_system`, we access the webview stored in
a bevy resource, and call `evaluate_script` on it with a Javascript snippet
built in-place based on the event type.

In `bridge`, we define the `wry_bridge` function. This function is used in the
`impl GetWindow for WryWebview` impl. Basically, whenever bevy creates a window,
the window will be converted into a webview, and `wry_bridge` will be added as
an `ipc_handler`.

`wry_bridge` will push `Request`s to a `Sender<Request>` based on what it got
from the IPC. This `Sender<Request>` is pased to `wry_bridge` via
a static variable, using a `OnceLock`. The `OnceLock` is initialized at app
startup, well before the first window is spawned.

Now, we can, from Javascript, call our custom protocol and send message to bevy.

And inversly, we can call Javascript from our rust code.

## Limitations

- Currently, since the webview and bevy share the same draw buffer, bevy will
  always draw on top of the webview
- The webview captures all input, making it impossible to react to input on the
  bevy-side of things.

## Future work

- Instead of creating javascript strings on the fly and executing them, we would
  using a serialization format such a JSON or MessagePack and call a singular
  Javascript function with it. It will be the job of the Javascript function to
  do anything based on the content of the JSON object.
- Getting input events to pass through the webview. Not sure what the way forward
  with this is.
  - The obvious solution, also very hacky, is to write a default javascript input
    handler that reacts to any uncaught bubbled-up events and communicate those
    input events to bevy through the IPC
  - The "correct" solution should probably work by setting the `transparent`
    field of the `WebViewBuilder`. Letting all uncaught events through.
  - When it comes to mouse picking, it is likely we would need to implement a
    `bevy_mod_picking` backend.
- The webview should actually be visible (of course)
  - The issue with the current setup is that bevy overwrites the buffer the webview
    writes to each frame.
  - Spawning the webview as a second window independent of bevy should also just work
