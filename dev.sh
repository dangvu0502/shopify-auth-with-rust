#!/bin/bash

# Run both frontend and backend in parallel

# Navigate to frontend directory and start Vite dev server
cd src/frontend
npm install
npm run dev &

# Navigate back to root and run Rust backend with auto-reloading
cd ..
cargo watch -x run
