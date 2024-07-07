# Rust remote conrol program:
Allows a computer to run axum based webserver on port 3000 to allow other devices to connect via browser to control parts of computer remotely, with features being:
* Volume control
* Media control
* Screen brightness control

Uses aesgcm encryption to send requests and recieve data and uses pbkdf2_sha256 hash algorithm to hash password.

Currently requires devices connecting to it to register url as use insecure as secure mode since crypto libaries for javascript require secure connection to work, since no https feature has been added yet.


Requirments:
* Rust
* Linux operating system
* playerctl
* pactl
* brightnessctl
* bash
* pamixer
