@echo on

cargo +nightly build --profile profiling

samply record target\profiling\game.exe
