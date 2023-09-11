check:
	cargo clippy
run:
	cargo run --bin spotlight -p spotlight --features bevy_winit_gtk/winit-gtk
run-winit:
	cargo run --bin spotlight -p spotlight --features bevy_winit_gtk/winit
