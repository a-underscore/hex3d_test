{
  description = "Vulkano Rust project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Pin a specific Rust toolchain via rust-toolchain.toml, or define one inline.
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" ];
        };

        # Vulkan + windowing native dependencies required by Vulkano.
        nativeBuildInputs = with pkgs; [
          rustToolchain
          pkg-config
          cmake          # some Vulkano transitive deps need it
        ];

        buildInputs = with pkgs; [
          # Vulkan loader & validation layers
          vulkan-loader
          vulkan-headers
          vulkan-validation-layers

          # Wayland windowing stack (winit wayland feature)
          wayland
          wayland-protocols
          libxkbcommon

          # shaderc / SPIR-V tools (needed if you compile GLSL at runtime)
          shaderc
          spirv-tools
        ];

        # Point the dynamic linker at the Vulkan loader and Wayland libs.
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
          vulkan-loader
          wayland
          libxkbcommon
        ]);

      in
      {
        # ── Development shell ────────────────────────────────────────────────
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;

          # Environment variables consumed by build scripts / Vulkano.
          VK_LAYER_PATH        = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          VULKAN_SDK           = "${pkgs.vulkan-headers}";
          # Force winit to use the Wayland backend.
          WINIT_UNIX_BACKEND   = "wayland";
          inherit LD_LIBRARY_PATH;

          shellHook = ''
            echo "🦀  Vulkano dev shell ready"
            echo "Rust: $(rustc --version)"
          '';
        };

        # ── Package (optional: builds the default binary) ────────────────────
        packages.default = (pkgs.makeRustPlatform {
          cargo = rustToolchain;
          rustc = rustToolchain;
        }).buildRustPackage {
          pname   = "vulkano-app";
          version = "0.1.0";
          src     = ./.;

          cargoLock.lockFile = ./Cargo.lock;

          inherit nativeBuildInputs buildInputs;
          inherit LD_LIBRARY_PATH;

          VK_LAYER_PATH      = "${pkgs.vulkan-validation-layers}/share/vulkan/explicit_layer.d";
          VULKAN_SDK         = "${pkgs.vulkan-headers}";
          WINIT_UNIX_BACKEND = "wayland";
        };

        # ── Apps shorthand ───────────────────────────────────────────────────
        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.default;
        };
      }
    );
}
