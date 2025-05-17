{ pkgs ? import <nixpkgs> {}}:
let 
    # latest
    esp-rs-src = builtins.fetchTarball "https://github.com/leighleighleigh/esp-rs-nix/archive/master.tar.gz";

    # pinned release
    #esp-rs-src = pkgs.fetchFromGitHub {
    #  owner = "leighleighleigh";
    #  repo = "esp-rs-nix";
    #  rev = "0c3fa7245d38019e60c4ae56b2e98465c1b8a5dc";
    #  hash = "sha256-b5kb6gxqutHySWEoweNfKbZw1r7DkwqRC39RWsyFSLU=";
    #};

    # earlier stable
    #esp-rs-src = pkgs.fetchFromGitHub {
    #  owner = "leighleighleigh";
    #  repo = "esp-rs-nix";
    #  rev = "8e3c612b3766385ebd5cd3f471f23a02e8382fdb";
    #  hash = "sha256-eOMxhDUapCt0/mUVONn4d8MJjUy6aYdbuxqHQseeab4=";
    #};

    esp-rs = pkgs.callPackage "${esp-rs-src}/esp-rs/default.nix" {};
    # source version
    #esp-rs = pkgs.callPackage "/home/leigh/Git/esp-rs-nix/esp-rs/default.nix" {};
in
pkgs.mkShell rec {
    name = "esp-rs-nix";

    buildInputs = [
        esp-rs 
        pkgs.rustup 
        #pkgs.espflash
        pkgs.rust-analyzer
        pkgs.rustfmt
        pkgs.rustc
        pkgs.clippy
        pkgs.pkg-config 
        pkgs.stdenv.cc 
        pkgs.bacon 
        pkgs.systemdMinimal
        pkgs.just 
        pkgs.lunarvim 
        pkgs.inotify-tools
        pkgs.picocom
        pkgs.vscode-fhs
        pkgs.libusb1
    ];

    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";

    shellHook = ''
    # custom bashrc stuff
    export PS1_PREFIX="(esp-rs)"
    . ~/.bashrc

    export LD_LIBRARY_PATH="''${LD_LIBRARY_PATH}:${LD_LIBRARY_PATH}"
    # this is important - it tells rustup where to find the esp toolchain,
    # without needing to copy it into your local ~/.rustup/ folder.
    export RUSTUP_TOOLCHAIN=${esp-rs}

    # Load shell completions for espflash
    if (which espflash >/dev/null 2>&1); then
    . <(espflash completions $(basename $SHELL))
    fi
    '';
}
