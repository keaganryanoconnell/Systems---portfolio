This is a systems engineering portfolio with 17 Rust crates and a Next.js 15 frontend.

Build: cargo check --workspace --all-features --exclude platform-nodes --exclude container-engine
Test: cargo test --workspace --all-features --exclude platform-nodes --exclude container-engine
Frontend: cd ui-control-center && npm run build && npm test
Live: https://systems-portfolio-five.vercel.app
