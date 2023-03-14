#!/usr/bin/env sh

echo "Welcome to wrap install script."

echo "Checking if Rust Lang is installed"

isRustInstalled=$(which cargo)

if which cargo ; then
    echo "Rust is installed"
else
    echo "Installing Rust Lang..."
    curl https://sh.rustup.rs -sSf | sh -s -- --help

    if which cargo ; then
        echo "Rust is installed"
    else
        echo "Something went wrong"
        exit 1
    fi
fi

# create foler for excutables
mkdir -p ~/bin
# create foler for compiling projects
mkdir -p ~/cli-projects && cd ~/cli-projects

rm -rf ~/cli-projects/wrap

echo ""
echo "Downloading 'wrap' project"
cargo new wrap
cd wrap

curl -O https://raw.githubusercontent.com/wormaga/wrap-solution/main/wrap/Cargo.toml
cd ./src
curl -O https://raw.githubusercontent.com/wormaga/wrap-solution/main/wrap/src/main.rs

echo ""
echo "Compiling 'wrap' project"
cd ~/cli-projects/wrap
cargo build --release

cp ~/cli-projects/wrap/target/release/wrap ~/bin

echo ""
echo "Ading folder with executables to PATH"
cd ~
# adding ~/bin foler to PATH
if [ -f ".bashrc" ]; then
  grep -qxF 'export PATH="${HOME}/bin:${PATH}"' ~/.bashrc || echo export PATH=\"\${HOME}/bin:\${PATH}\"  >> ~/.bashrc
fi
if [ -n "$BASH_VERSION" ]; then
  source ~/.bashrc
fi

if [ -f ".zshrc" ]; then
  grep -qxF 'export PATH="${HOME}/bin:${PATH}"' ~/.zshrc || echo export PATH=\"\${HOME}/bin:\${PATH}\"  >> ~/.zshrc
fi
if [ -n "$ZSH_VERSION" ]; then
  source ~/.zshrc
fi

# execute 'wrap' cli tool, that will install all other tools
echo "Starting 'wrap' program"
~/bin/wrap



echo ""
echo "Exiting at $(date)"
echo ""