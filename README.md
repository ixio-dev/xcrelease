# xcrelease

A command-line tool for automating the release process of Xcode projects with Sparkle framework support.

## Features

- Automates Xcode project archiving and exporting
- Creates DMG files for distribution
- Handles Apple notarization and stapling
- Generates Sparkle appcast files
- Supports version incrementing (major, minor, patch)
- Dry-run mode for testing without executing actions

## Installation

```bash
# Clone the repository
git clone https://github.com/your-repo/xcrelease.git
cd xcrelease

# Build and install
cargo install --path .
```

## Setup

1. Create a `.deployment` file in your Xcode project directory based on the template:

```bash
xcrelease template
```

2. Configure the required parameters in `.deployment`:

```
APP_NAME=MyApp
APPLE_ID=email@example.com
TEAM_ID=XXXXXXXXXX
ACCOUNT_TOKEN=xxxx-xxxx-xxxx-xxxx
SCHEME=MyApp
EXPORT_METHOD=developer-id
```

## Usage

```bash
# Release with incremented patch version
xcrelease release --patch

# Release with incremented minor version
xcrelease release --minor

# Release with incremented major version
xcrelease release --major

# Release with a specific version
xcrelease release --ver 2.0.0

# Dry run to see what would be executed
xcrelease --dry-run release --patch
```

## Commands

- `completion`: Generate shell completion scripts
- `template`: Show .deployment file template
- `release`: Release the Xcode project with optional version incrementing

## Requirements

- macOS with Xcode installed
- Apple Developer account
- Xcode project configured for code signing
- A `.deployment` file in the project directory