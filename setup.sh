#!/bin/bash

Distro="$(lsb_release -i | cut -f 2-)"

echo "Do you want to launch the autoclicker after install? (y/N)"
read LaunchAfterInstall

# Check if we can install dependencies
if [ "$Distro" == "Arch" ]; then
  echo "Do you want to automatically install tauri? (Y/n)"
  read InstallTauri

  case "$InstallTauri" in
  "" | "Y" | "y")
    echo "Installing tauri..."
    sudo pacman -S --needed \
      webkit2gtk-4.1 \
      base-devel \
      curl \
      wget \
      file \
      openssl \
      appmenu-gtk-module \
      libappindicator-gtk3 \
      librsvg \
      xdotool
    ;;
  "N" | "n")
    echo "Skipping to normal setup. (ENSURE YOU HAVE TAURI)"
    ;;
  esac

elif [ "$Distro" == "Ubuntu" ]; then
  echo "Do you want to automatically install tauri? (Y/n)"
  read InstallTauri

  case "$InstallTauri" in
  "" | "Y" | "y")
    echo "Installing tauri..."
    sudo apt install libwebkit2gtk-4.1-dev \
      build-essential \
      curl \
      wget \
      file \
      libxdo-dev \
      libssl-dev \
      libayatana-appindicator3-dev \
      librsvg2-dev
    ;;
  "N" | "n")
    echo "Skipping to normal setup. (ENSURE YOU HAVE TAURI)"
    ;;
  esac

else
  echo "Unable to automatically install Tauri (Unsupported OS, Supported OSes include Arch Linux, Ubuntu)"
fi

echo "Installing nessesary packages..."
npm install

echo "Seting toolchain..."
rustup default stable

echo "Building production build..."
npm exec tauri build

echo "Creating CLI shortcut..."
echo -e "cd /\n.$(pwd)/src-tauri/target/release/BlurAutoClicker &" >>openClicker.sh
chmod +x openClicker.sh
sudo mv openClicker.sh /usr/local/bin
alias clicker="openClicker.sh"

case "$LaunchAfterInstall" in
"Y" | "y")
  echo "Launching..."
  if [ -e "clicker" ]; then
    clicker
  fi
  ;;
"" | "N" | "n")
  echo "To launch run 'clicker', If clicking doesn't work try restarting or adding yourself to the input group"
  ;;
esac
