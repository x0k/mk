#!/usr/bin/env bash

t:
    # Run tests
    cargo test

b:
    # Build binary
    cargo build

h:
    # Show help
    mk --printer targets "*"
