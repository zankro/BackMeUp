#!/bin/bash

# Determina l'utente corrente
if [ "$EUID" -eq 0 ]; then
  # Se sei root, usa SUDO_USER o un comando di fallback
  CURRENT_USER="${SUDO_USER:-$(who | awk '{print $1}' | head -n 1)}"
else
  # Se non sei root, usa whoami
  CURRENT_USER=$(whoami)
fi

echo "Utente rilevato: $CURRENT_USER"

# Verifica se l'utente è stato trovato
if [ -z "$CURRENT_USER" ]; then
  echo "Errore: impossibile determinare l'utente."
  exit 1
fi

# Percorso principale della cartella BackMeUp
BASE_DIR="$(dirname "$(realpath "$0")")/BackMeUp"
TARGET_DIR="$BASE_DIR/../target/debug"
ASSETS_DIR="$BASE_DIR/../assets"

# Creazione delle cartelle necessarie
echo "Creazione della struttura delle cartelle..."
mkdir -p "$BASE_DIR/Resources/audio"
mkdir -p "$BASE_DIR/bin"

# Copia dei file audio dalla cartella assets nella cartella Resources/audio
echo "Copia dei file audio..."
cp "$ASSETS_DIR/success-48018.mp3" "$BASE_DIR/Resources/audio/"
cp "$ASSETS_DIR/stop-13692.mp3" "$BASE_DIR/Resources/audio/"
cp "$ASSETS_DIR/blip-131856.mp3" "$BASE_DIR/Resources/audio/"

# Esecuzione di cargo build
echo "Compilazione del progetto Rust..."
cd "$BASE_DIR"
cd ..
cargo build

# Verifica della compilazione e copia degli eseguibili
if [ -f "$TARGET_DIR/Group16" ]; then
  echo "Copia degli eseguibili nella cartella bin..."
  cp "$TARGET_DIR/Group16" "$BASE_DIR/bin/"
  cp "$TARGET_DIR/backup_program" "$BASE_DIR/bin/"
  cp "$TARGET_DIR/config_program" "$BASE_DIR/bin/"
  cp "$TARGET_DIR/service" "$BASE_DIR/bin/"
  cp "$TARGET_DIR/uninstall_service" "$BASE_DIR/bin/"
else
  echo "Errore: La compilazione del progetto non è riuscita."
  exit 1
fi

echo "Done"

# Avvia il programma
echo "Avvio dell'applicazione..."
"$BASE_DIR/bin/Group16"
echo "Installazione completata con successo!"
