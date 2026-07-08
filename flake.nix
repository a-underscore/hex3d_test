{
  description = "Vulkano development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "clippy"
          ];
        };
      in {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
	    gcc
            rustToolchain

            pkg-config
            cmake

            vulkan-loader
            vulkan-validation-layers
            vulkan-headers

            shaderc
            spirv-tools

            wayland
            wayland-protocols
            libxkbcommon

            xorg.libX11
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ];

          VK_LAYER_PATH =
            "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
	  SHADERC_LIB_DIR = "${pkgs.shaderc.lib}/lib";

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
            pkgs.vulkan-loader
            pkgs.shaderc
            pkgs.wayland
            pkgs.libxkbcommon
            pkgs.xorg.libX11
          ];

          shellHook = ''
            echo "🦀 Vulkano development shell"
            rustc --version
          '';
        };
      });
}
