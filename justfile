set shell := ["cmd.exe", "/c"]

default-features := ""
default-run := "outputs/test_program_rust.exe"

run-server-internal pwd="" run=default-run features=default-features:
    cd ./crates/payload_server && cargo run --release --features={{features}} -- --stage2 ../../outputs/stage2.bin --run {{pwd}}/{{run}}

run-server run=default-run features=default-features:
    just --justfile {{justfile()}} run-server-internal %PWD% {{run}} {{features}}

build-stage1 features=default-features:
    cd ./crates/shellcode_stage1 && cargo build --release --features={{features}}

build-stage1-network features=default-features:
    cd ./crates/shellcode_stage1_network && cargo build --release --features={{features}}

build-stage2 features=default-features:
    cd ./crates/shellcode_stage2 && cargo build --release --features={{features}}

build-test-program-rust features=default-features:
    cd ./crates/test_program_rust && cargo build --release --features={{features}}
    cp ./crates/test_program_rust/target/release/test_program_rust.exe ./outputs/

build-exploit features=default-features:
    just --justfile {{justfile()}} build-stage1 {{features}}
    just --justfile {{justfile()}} build-stage1-network {{features}}
    just --justfile {{justfile()}} build-stage2 {{features}}
    cd ./crates/shellcode_gen && cargo run --release -- ../../outputs/

generate features=default-features:
    just --justfile {{justfile()}} build-exploit {{features}}
    just --justfile {{justfile()}} build-test-program-rust {{features}}
    xcopy /y crates\\shellcode_stage2\\target\\x86_64-pc-windows-msvc\\release\\shellcode_stage2.bin %LOCALAPPDATA%\\Packages\\27878ConstantineTarasenko.458004FD2C47C_c8b3w9r5va522\\LocalState\\stage2.bin
    xcopy /y crates\\test_program_rust\\target\\release\\test_program_rust.exe %LOCALAPPDATA%\\Packages\\27878ConstantineTarasenko.458004FD2C47C_c8b3w9r5va522\\LocalState
    xcopy /y outputs\\gamescript_autosave.txt %LOCALAPPDATA%\\Packages\\27878ConstantineTarasenko.458004FD2C47C_c8b3w9r5va522\\LocalState\\gamescript_autosave.txt

generate-dev features=default-features:
    just --justfile {{justfile()}} build-exploit {{features}}
    just --justfile {{justfile()}} build-test-program-rust network,{{features}}
    xcopy /y crates\\shellcode_stage2\\target\\x86_64-pc-windows-msvc\\release\\shellcode_stage2.bin %LOCALAPPDATA%\\Packages\\27878ConstantineTarasenko.458004FD2C47C_c8b3w9r5va522\\LocalState\\stage2.bin
    xcopy /y crates\\test_program_rust\\target\\release\\test_program_rust.exe %LOCALAPPDATA%\\Packages\\27878ConstantineTarasenko.458004FD2C47C_c8b3w9r5va522\\LocalState
    xcopy /y outputs\\gamescript_autosave_network.txt %LOCALAPPDATA%\\Packages\\27878ConstantineTarasenko.458004FD2C47C_c8b3w9r5va522\\LocalState\\gamescript_autosave.txt
