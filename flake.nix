{
  description = "Pepometer: Tauri 2 + Rust + Svelte dev shell with the Android toolchain";

  inputs = {
    # Pinned with BaseHero's Tauri Android spike so neptune's store already has
    # the Android SDK, NDK and emulator image.
    nixpkgs.url = "github:nixos/nixpkgs/c59305bab2065cfecc4944690d9eedbb56f3a9fa";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
  }: let
    system = "x86_64-linux";
    pkgs = import nixpkgs {
      inherit system;
      overlays = [rust-overlay.overlays.default];
      config = {
        allowUnfree = true;
        android_sdk.accept_license = true;
      };
    };

    rust = pkgs.rust-bin.stable.latest.default.override {
      extensions = ["rust-src" "clippy" "rustfmt"];
      targets = [
        "x86_64-unknown-linux-gnu"
        "aarch64-linux-android"
        "x86_64-linux-android"
        "armv7-linux-androideabi"
        "i686-linux-android"
      ];
    };

    ndkVersion = "27.2.12479018";
    android = pkgs.androidenv.composeAndroidPackages {
      platformVersions = ["34" "35" "36"];
      buildToolsVersions = ["35.0.0" "36.0.0"];
      includeNDK = true;
      ndkVersions = [ndkVersion];
      includeEmulator = true;
      includeSystemImages = true;
      systemImageTypes = ["google_apis_playstore"];
      abiVersions = ["x86_64"];
      cmakeVersions = ["3.22.1"];
    };
    sdk = "${android.androidsdk}/libexec/android-sdk";

    # Developer sign-in helper. python-garminconnect's multi-strategy login is
    # the most reliable way to sign in from a computer; the CLI imports the
    # token file it writes. nixpkgs' garminconnect is too old (pre-0.3).
    py = pkgs.python312Packages;
    uaGenerator = py.buildPythonPackage rec {
      pname = "ua-generator";
      version = "2.0.26";
      pyproject = true;
      src = pkgs.fetchPypi {
        pname = "ua_generator";
        inherit version;
        sha256 = "0blxn67y01yg8l29rzj0ajnxwfvv77cvl4g7f6xy3kr90pc3ksci";
      };
      build-system = [py.setuptools];
      doCheck = false;
    };
    garminconnect = py.buildPythonPackage rec {
      pname = "garminconnect";
      version = "0.3.18";
      pyproject = true;
      src = pkgs.fetchFromGitHub {
        owner = "cyberjunky";
        repo = "python-garminconnect";
        rev = version;
        hash = "sha256-YT6keE60qlsP3Z58UqBhjgBSUCld5KEDNtK/4MS/qY4=";
      };
      build-system = [py.pdm-backend];
      dependencies = [py.curl-cffi py.requests uaGenerator];
      pythonRelaxDeps = true;
      doCheck = false;
      pythonImportsCheck = ["garminconnect"];
    };
    garminLogin = pkgs.writeShellApplication {
      name = "pepometer-garmin-login";
      runtimeInputs = [(pkgs.python312.withPackages (_: [garminconnect]))];
      text = ''
        out="''${PEPOMETER_HOME:-$PWD/.data}"
        mkdir -p "$out"
        chmod 700 "$out"
        exec python3 ${./scripts/garmin-login.py} "$out"
      '';
    };

    # Tauri's desktop runtime links WebKitGTK; needed to build and test the
    # app crate on the host.
    linuxDeps = with pkgs; [
      webkitgtk_4_1
      gtk3
      libsoup_3
      glib
      glib-networking
      cairo
      pango
      gdk-pixbuf
      atk
      librsvg
      openssl
      dbus
      libayatana-appindicator
    ];
  in {
    packages.${system}.garmin-login = garminLogin;
    apps.${system}.garmin-login = {
      type = "app";
      program = "${garminLogin}/bin/pepometer-garmin-login";
    };

    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        rust
        pkg-config
        cargo-tauri
        nodejs_22
        jdk17
        android.androidsdk
        sqlite
        imagemagick
        jq
        gnumake
      ];
      buildInputs = linuxDeps;

      ANDROID_HOME = sdk;
      ANDROID_SDK_ROOT = sdk;
      NDK_HOME = "${sdk}/ndk/${ndkVersion}";
      JAVA_HOME = pkgs.jdk17.home;
      # Gradle downloads a dynamically linked aapt2; use the SDK's patched one.
      GRADLE_OPTS = "-Dorg.gradle.project.android.aapt2FromMavenOverride=${sdk}/build-tools/35.0.0/aapt2";

      shellHook = ''
        export GIO_MODULE_DIR=${pkgs.glib-networking}/lib/gio/modules
        export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
        export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath linuxDeps}:$LD_LIBRARY_PATH
        export ANDROID_AVD_HOME=$PWD/.android-avd
        export ANDROID_USER_HOME=$PWD/.android-user
        # Pepometer's emulator runs on its own port; other emulators may be up.
        export ANDROID_SERIAL=''${ANDROID_SERIAL:-emulator-5560}
      '';
    };
  };
}
