cargo new <name>

cargo build

cargo run

cargo build --release

mkdir -p ${HOME}/bin

cp target/release/litegallery ${HOME}/bin/litegallery

chmod 755 \${HOME}/bin/litegallery

//copy this:

//export PATH="\${HOME}/bin:${PATH}"

nano \${HOME}/.zshrc

nano \${HOME}/.bashrc

############################

rustup target list

rustup target add aarch64-apple-darwin

rustup target add x86_64-apple-darwin

cargo build --target aarch64-apple-darwin

cargo build --target x86_64-apple-darwin

############################