{
  # this had to be made because FUCK docker
  description = "development environment for bbg";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay/master";
  };

  outputs = { self, nixpkgs, rust-overlay }: let
    pkgs = import nixpkgs {
      overlays = [ rust-overlay.overlay ];
    };
  in
  {
    devShells.default = pkgs.mkShell {
      buildInputs = [
        pkgs.rustc
        pkgs.cargo
        pkgs.pkg-config     
        pkgs.openssl        
        pkgs.zlib           
        pkgs.libpng         
        pkgs.freetype       
        pkgs.fontconfig     
      ];

      RUSTFLAGS = "-C target-cpu=native";

      shellHook = ''
        echo "welcome to the bbg dev shell!"
        echo "rust version: $(rustc --version)"
        echo "cargo version: $(cargo --version)"
      '';
    };
  };
}

# wheres the shell.nix??
# ...
