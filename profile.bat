@echo on

cargo +nightly build --profile profiling

samply record target\profiling\backrooms_liminal.exe
