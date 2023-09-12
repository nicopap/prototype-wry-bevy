check:
	cargo clippy
run:
	cargo run --bin wry_demo -p wry_demo --features bevy_winit_gtk/winit-gtk
run-winit:
	cargo run --bin wry_demo -p wry_demo --features bevy_winit_gtk/winit
