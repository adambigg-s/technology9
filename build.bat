@echo on

cargo +nightly build --release

copy target\release\game.exe .
