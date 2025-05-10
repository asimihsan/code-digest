setup:
    mise trust
    mise install
    cargo binstall -y cargo-checkmate
    cargo binstall -y cargo-audit
    cargo binstall -y dioxus-cli

serve:
    (cd ui && dx serve --platform desktop)
