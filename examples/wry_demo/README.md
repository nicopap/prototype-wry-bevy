# Bevy wry demo

Shows 2-ways communication between a webpage running in a webview and bevy.

## How it works

It's extremely dumb.

In `bridge` we define two enums. `Request` are events sent from the Javascript
world to the bevy world, while `Event` are events going from bevy to Javascript.

In `bridge` we define the `bevy_bridge_system`, a bevy system that reads
events from a `Receiver<Request>`, run bevy-related code and "answers" to
the Javascript world by calling `to_wry` with an `Event`. `to_wry` takes the
webview from a resource and runs some Javascript code based on the event.

In `bridge`, we define the `wry_bridge` function. This function is used in the
`impl GetWindow for WryWebview` impl. Basically, whenever bevy creates a window,
it will call this implementation, and add `wry_bridge` as a custom protocol handler.

`wry_bridge` will push `Request`s to a `Sender<Request>` based on what it got
from the custom protocol. This `Sender<Request>` is pased to `wry_bridge` via
a static variable, using a `OnceLock`. The `OneLock` is initialized at app
startup, well before the first window is spawned.

Now, we can, from Javascript, call our custom protocol and send message to bevy.

And inversly, we can call Javascript from our rust code.

## Possible improvements

- Generating on the go javascript code seems less than idea:
  - reeks of potential XSS attacks
  - Would require an allocation per event
- Not much control over bevy
  - I'm thinking we could probably write wrapper around bevy commands
  - For queries, well, that's for another milestone ;)
- Webview is not visible. We are waiting on wusyong's updates on `winit_gtk`
  for this. Right now, we are using `bevy_tao` in order to be able to play with
  `wry`.

Lead to fix that is to use the `RequestAsyncResponder` to push responses to
Javascript through the custom protocol.