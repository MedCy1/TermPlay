#!/bin/bash

# Release script for TermPlay
# Usage: ./scripts/release.sh [patch|minor|major]

set -e

# Colors for messages
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Utility functions
log_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

log_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

log_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

log_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Prerequisite checks
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if we are in a Git repo
    if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
        log_error "This script must be run inside a Git repository"
        exit 1
    fi
    
    # Check if we are on the main branch
    current_branch=$(git branch --show-current)
    if [ "$current_branch" != "main" ]; then
        log_error "This script must be run on the 'main' branch. Current branch: $current_branch"
        exit 1
    fi
    
    # Check if working directory is clean
    if ! git diff-index --quiet HEAD --; then
        log_error "Working directory is not clean. Commit your changes before making a release."
        exit 1
    fi
    
    # Check if cargo is installed
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed"
        exit 1
    fi
    
    # Check if gh CLI is installed (optional)
    if ! command -v gh &> /dev/null; then
        log_warning "GitHub CLI (gh) is not installed. GitHub releases will need to be created manually."
    fi
    
    log_success "Prerequisites checked"
}

# Get current version
get_current_version() {
    grep "^version" Cargo.toml | cut -d'"' -f2
}

# Calculate new version
calculate_new_version() {
    local current_version=$1
    local bump_type=$2
    
    # Validate version format
    if [[ ! $current_version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        log_error "Invalid version format: $current_version"
        exit 1
    fi
    
    IFS='.' read -ra VERSION_PARTS <<< "$current_version"
    major=${VERSION_PARTS[0]:-0}
    minor=${VERSION_PARTS[1]:-0}
    patch=${VERSION_PARTS[2]:-0}
    
    case $bump_type in
        major)
            major=$((major + 1))
            minor=0
            patch=0
            ;;
        minor)
            minor=$((minor + 1))
            patch=0
            ;;
        patch)
            patch=$((patch + 1))
            ;;
        *)
            log_error "Invalid bump type: $bump_type. Use: patch, minor, or major"
            exit 1
            ;;
    esac
    
    echo "$major.$minor.$patch"
}

# Update version in Cargo.toml
update_version() {
    local new_version=$1
    log_info "Updating version to $new_version..."
    
    # Update with sed (compatible macOS and Linux)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    else
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml
    fi
    
    log_success "Version updated in Cargo.toml"
}

# Generate changelog
generate_changelog() {
    local version=$1
    local changelog_file="CHANGELOG.md"
    
    log_info "Generating changelog..."
    
    # Create CHANGELOG.md if it doesn't exist
    if [ ! -f "$changelog_file" ]; then
        echo "# Changelog" > "$changelog_file"
        echo "" >> "$changelog_file"
        echo "All notable changes to this project will be documented in this file." >> "$changelog_file"
        echo "" >> "$changelog_file"
    fi
    
    # Get commits since last tag
    local last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    local commits
    
    if [ -n "$last_tag" ]; then
        commits=$(git log --oneline --pretty=format:"- %s (%h)" "$last_tag"..HEAD)
    else
        commits=$(git log --oneline --pretty=format:"- %s (%h)")
    fi
    
    # Add new version to changelog
    local temp_file=$(mktemp)
    {
        echo "# Changelog"
        echo ""
        echo "## [$version] - $(date +%Y-%m-%d)"
        echo ""
        if [ -n "$commits" ]; then
            echo "$commits"
        else
            echo "- Minor fixes and improvements"
        fi
        echo ""
        tail -n +3 "$changelog_file"
    } > "$temp_file"
    
    mv "$temp_file" "$changelog_file"
    log_success "Changelog generated"
}

# Build and test
build_and_test() {
    log_info "Building and running tests..."
    
    # Update Cargo.lock
    cargo check
    
    # Run tests
    cargo test --release
    
    # Build release
    cargo build --release
    
    # Check if binary works
    if ./target/release/termplay --version > /dev/null; then
        log_success "Build and tests successful"
    else
        log_error "Release binary does not work correctly"
        exit 1
    fi
}

# Create commit and tag
create_git_tag() {
    local version=$1
    
    log_info "Creating Git commit and tag..."
    
    # Add modified files
    git add Cargo.toml Cargo.lock CHANGELOG.md
    
    # Create commit
    git commit -m "chore: release version $version"
    
    # Create tag
    git tag -a "v$version" -m "Release version $version"
    
    log_success "Tag v$version created"
}

# Push to GitHub
push_to_github() {
    local version=$1
    
    log_info "Pushing to GitHub..."
    
    # Push branch and tags
    git push origin main
    git push origin "v$version"
    
    log_success "Pushed to GitHub"
}

# Create GitHub release
create_github_release() {
    local version=$1
    
    if command -v gh &> /dev/null; then
        log_info "Creating GitHub release..."
        
        # Extract release notes from changelog
        local release_notes=$(sed -n "/## \[$version\]/,/## \[/p" CHANGELOG.md | sed '$d' | tail -n +3)
        
        # Create release
        echo "$release_notes" | gh release create "v$version" \
            --title "Release $version" \
            --notes-file - \
            --draft
        
        log_success "GitHub release created (draft)"
        log_info "Visit https://github.com/$(gh repo view --json owner,name -q '.owner.login + \"/\" + .name')/releases to publish the release"
    else
        log_warning "GitHub CLI not available. Create the release manually on GitHub."
    fi
}

# Main script
main() {
    local bump_type=${1:-patch}
    
    echo "🚀 TermPlay Release Script"
    echo "============================="
    
    check_prerequisites
    
    local current_version=$(get_current_version)
    local new_version=$(calculate_new_version "$current_version" "$bump_type")
    
    log_info "Current version: $current_version"
    log_info "New version: $new_version"
    
    # Ask for confirmation
    echo ""
    read -p "Continue with release $new_version? (y/N) " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Release cancelled"
        exit 0
    fi
    
    # Release process
    update_version "$new_version"
    generate_changelog "$new_version"
    build_and_test
    create_git_tag "$new_version"
    push_to_github "$new_version"
    create_github_release "$new_version"
    
    echo ""
    log_success "🎉 Release $new_version completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Check that the GitHub Actions pipeline runs correctly"
    echo "2. Test the generated binaries"
    echo "3. Publish the draft release on GitHub"
    echo "4. Announce the release to your community"
}

# Show help if requested
if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
    echo "Usage: $0 [patch|minor|major]"
    echo ""
    echo "Release types:"
    echo "  patch  - Bug fixes (1.0.0 -> 1.0.1)"
    echo "  minor  - New features (1.0.0 -> 1.1.0)"
    echo "  major  - Breaking changes (1.0.0 -> 2.0.0)"
    echo ""
    echo "Default: patch"
    exit 0
fi

# Run main script
main "$@"