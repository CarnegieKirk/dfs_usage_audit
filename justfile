# use PowerShell instead of sh:
set shell := ["pwsh.exe", "-NoProfile", "-c"]

default_opts := "-- --path \"S:\\\" -D"

hello:
  Write-Host "Hello, world!"

build:
  cargo build --release

fast:
  cargo run --release  {{default_opts}}

slow:
  cargo run {{default_opts}}
