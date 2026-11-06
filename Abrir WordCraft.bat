@echo off
setlocal
cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 (
    echo Rust Cargo nao foi encontrado.
    echo Abra o Developer PowerShell do Visual Studio e confira a instalacao do Rust.
    pause
    exit /b 1
)

echo Conferindo e compilando o WordCraft...
cargo build --release -p wordcraft
if errorlevel 1 (
    echo A compilacao falhou. Leia a mensagem acima para ver o erro.
    pause
    exit /b 1
)

if not exist "%~dp0target\release\wordcraft.exe" (
    echo O executavel nao foi encontrado em target\release.
    pause
    exit /b 1
)

start "" "%~dp0target\release\wordcraft.exe"
exit /b 0
