# Bevy wry demo


Shows 2-ways communication between a webpage running in a webview and bevy.

## What it is

- The webview navigates to `https://bevyengine.org`.
- A javascript snippet reads all the links on the page and spawns a sprite in bevy per link.
- Bevy highlights a different sprite each 1.2 second, showing the associated link
- When pressing space bar, bevy will send a request to the webview to move to the highlighted link
- Whenever navigating to a new page, the sprites will be replaced by new ones
  corresponding to the links on the new page (yay hypertext)

This is mostly a copy of `contributors.rs` from the bevy repository.

## How it works

There are a few tricks to get this hack to work:

1. A fork of `bevy_winit` that uses `tao` instead of `winit`
2. We replace the default bevy loop runner by a loop that spawns a webview as
   an independent window and store it as a resource.
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

In `bridge`, we define the `wry_bridge` function. This function is used as IPC handler in the
`setup_webview`, the function that spawns the webview before the runner.

`wry_bridge` will push `Request`s to a `Sender<Request>` based on what it got
from the IPC.

Now, we can, from Javascript, call our custom protocol and send message to bevy.

And inversly, we can call Javascript from our rust code.

## Limitations

- When the webview is focused, it captures all input, making it impossible to
  react to input on the bevy-side of things.
- Right now, the webview window is independent from the bevy window, so it's easy
  to just move one and leave the other behind :P
- We don't handle the window spawned by `setup_webview` on the bevy side, which results in a
  lot of error logs.

## Future work

- Instead of creating javascript strings on the fly and executing them, we would
  using a serialization format such a JSON or MessagePack and call a singular
  Javascript function with it. It will be the job of the Javascript function to
  do anything based on the content of the JSON object.
- Getting input events to pass through the webview. Not sure what the way forward
  with this is.
- The webview should most likely be part of the bevy window, instead of an independent one
  - The goal of the `winit_gtk` fork is to enable this specifically.
  - Otherwise, the "trick" would be to sync all window property between the bevy window
    and the webview window.
