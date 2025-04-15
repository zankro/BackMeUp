#!/bin/bash

# Funzione per rilevare il sistema operativo
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if grep -q Microsoft /proc/version &>/dev/null; then
            echo "windows" # WSL (Windows Subsystem for Linux)
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unsupported"
    fi
}

# Funzione per Windows tramite WSL
setup_windows() {
    echo "Configurazione per Windows tramite WSL..."

    BASE_DIR="$(pwd)/BackMeUp"
    TARGET_DIR="$(pwd)/target/debug"
    ASSETS_DIR="$(pwd)/assets"

    mkdir -p "$BASE_DIR/Resources/audio"
    mkdir -p "$BASE_DIR/bin"

    echo "Copia dei file audio..."
    cp -f "$ASSETS_DIR/success-48018.mp3" "$BASE_DIR/Resources/audio/"
    cp -f "$ASSETS_DIR/stop-13692.mp3" "$BASE_DIR/Resources/audio/"
    cp -f "$ASSETS_DIR/blip-131856.mp3" "$BASE_DIR/Resources/audio/"

    echo "Compilazione del progetto Rust..."
    cargo build

    if [ -f "$TARGET_DIR/Group16.exe" ]; then
        echo "Copia degli eseguibili nella cartella bin..."
        cp -f "$TARGET_DIR/Group16.exe" "$BASE_DIR/bin/"
    else
        echo "Errore: La compilazione del progetto non è riuscita."
        exit 1
    fi

    echo "Avvio dell'applicazione..."
    cd "$BASE_DIR/bin" || exit
    ./Group16.exe
}

# Funzione per macOS o Linux
setup_unix() {
    echo "Configurazione per macOS/Linux..."

    BASE_DIR="$(pwd)/BackMeUp"
    TARGET_DIR="$(pwd)/target/debug"
    ASSETS_DIR="$(pwd)/assets"

    mkdir -p "$BASE_DIR/Resources/audio"
    mkdir -p "$BASE_DIR/bin"

    echo "Copia dei file audio..."
    cp -f "$ASSETS_DIR/success-48018.mp3" "$BASE_DIR/Resources/audio/"
    cp -f "$ASSETS_DIR/stop-13692.mp3" "$BASE_DIR/Resources/audio/"
    cp -f "$ASSETS_DIR/blip-131856.mp3" "$BASE_DIR/Resources/audio/"

    echo "Compilazione del progetto Rust..."
    cargo build

    if [ -f "$TARGET_DIR/Group16" ]; then
        echo "Copia degli eseguibili nella cartella bin..."
        cp -f "$TARGET_DIR/Group16" "$BASE_DIR/bin/"
    else
        echo "Errore: La compilazione del progetto non è riuscita."
        exit 1
    fi

    echo "Avvio dell'applicazione..."
    cd "$BASE_DIR/bin" || exit
    ./Group16 &
}

# Main script
os=$(detect_os)

case "$os" in
    windows)
        setup_windows
        ;;
    linux|macos)
        setup_unix
        ;;
    *)
        echo "Sistema operativo non supportato."
        exit 1
        ;;
esac

echo "Installazione completata con successo!"
