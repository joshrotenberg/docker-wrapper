#!/bin/bash
# ADR Helper Script v2.0 - Enhanced with Authorization & Multi-Developer Support

set -e

CLAUDE_DIR=".claude"
INDEX_FILE="$CLAUDE_DIR/adr-index.toml"

# Color output for better UX
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 🔐 Permission checking functions
check_permission() {
    local operation="$1"
    local permission_key="$2"
    local default="${3:-ask}"

    if [[ ! -f "$INDEX_FILE" ]]; then
        echo -e "${YELLOW}Warning: No ADR index found. Using safe defaults.${NC}"
        permission="ask"
    else
        permission=$(grep "^$permission_key" "$INDEX_FILE" 2>/dev/null | cut -d'"' -f2 || echo "$default")
    fi

    case "$permission" in
        "never")
            echo -e "${RED}❌ Operation '$operation' is disabled by configuration${NC}"
            return 1
            ;;
        "ask")
            echo -e "${YELLOW}🤔 Permission required for: $operation${NC}"
            read -p "Proceed? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                return 0
            else
                echo -e "${YELLOW}⏭️  Operation cancelled by user${NC}"
                return 1
            fi
            ;;
        "yes")
            echo -e "${GREEN}✅ Auto-approved: $operation${NC}"
            return 0
            ;;
        *)
            echo -e "${YELLOW}⚠️  Unknown permission '$permission', asking for safety${NC}"
            read -p "Proceed with '$operation'? (y/N): " -n 1 -r
            echo
            [[ $REPLY =~ ^[Yy]$ ]]
            ;;
    esac
}

# Git operation wrapper
safe_git() {
    local operation="$1"
    local permission_key="$2"
    shift 2

    if check_permission "$operation" "$permission_key"; then
        echo -e "${BLUE}🔧 Executing: git $*${NC}"
        git "$@"
    else
        return 1
    fi
}

# GitHub CLI operation wrapper
safe_gh() {
    local operation="$1"
    local permission_key="$2"
    shift 2

    if ! command -v gh &> /dev/null; then
        echo -e "${YELLOW}⚠️  GitHub CLI not found. Install 'gh' for GitHub operations.${NC}"
        return 1
    fi

    if check_permission "$operation" "$permission_key"; then
        echo -e "${BLUE}🔧 Executing: gh $*${NC}"
        gh "$@"
    else
        return 1
    fi
}

validate() {
    echo "Validating ADR structure..."

    if [[ ! -f "$INDEX_FILE" ]]; then
        echo "ERROR: ADR index not found at $INDEX_FILE"
        exit 1
    fi

    # Check if all referenced ADR files exist
    if command -v grep &> /dev/null; then
        while IFS= read -r line; do
            if [[ $line =~ file[[:space:]]*=[[:space:]]*\"([^\"]+)\" ]]; then
                adr_file="$CLAUDE_DIR/branches/${BASH_REMATCH[1]}"
                if [[ ! -f "$adr_file" ]]; then
                    echo "ERROR: Missing ADR file: $adr_file"
                    exit 1
                fi
            fi
        done < "$INDEX_FILE"
    fi

    echo "Validation complete"
}

list() {
    echo "Active ADRs:"
    echo ""

    if [[ -f "$INDEX_FILE" ]]; then
        grep -E '^".*" = \{' "$INDEX_FILE" | \
        sed 's/"//g' | sed 's/ = {//' | \
        while read -r adr; do
            echo "  • $adr"
        done
    else
        echo "No ADR index found"
    fi
}

status() {
    echo "ADR System Status:"
    echo ""

    if [[ -f "$INDEX_FILE" ]]; then
        active_count=$(grep -c '\[active\.' "$INDEX_FILE" 2>/dev/null || echo "0")
        echo "  Active ADR sections: $active_count"

        total_adrs=$(grep -c '^".*" = {' "$INDEX_FILE" 2>/dev/null || echo "0")
        echo "  Total ADRs: $total_adrs"

        if [[ -d "$CLAUDE_DIR/merged" ]]; then
            merged_count=$(find "$CLAUDE_DIR/merged" -name "*.md" 2>/dev/null | wc -l)
            echo "  Merged ADRs: $merged_count"
        fi

        echo ""
        echo "  Categories:"
        grep '\[active\.' "$INDEX_FILE" 2>/dev/null | sed 's/\[active\./  • /' | sed 's/\]//' || echo "  None"
    else
        echo "ERROR: No ADR index found"
    fi
}

new_adr() {
    local category="${1:-feat}"
    local title="${2:-new-decision}"
    local sanitized_title=$(echo "$title" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g' | sed 's/^-\|-$//g')
    local adr_file="$CLAUDE_DIR/branches/$category/$sanitized_title.md"
    local template_file="$CLAUDE_DIR/templates/adr-template.md"

    if [[ -f "$adr_file" ]]; then
        echo "ERROR: ADR already exists: $adr_file"
        exit 1
    fi

    mkdir -p "$(dirname "$adr_file")"

    if [[ -f "$template_file" ]]; then
        cp "$template_file" "$adr_file"
    else
        cat > "$adr_file" << EOF
# ADR: [Decision Title]

**Status**: Proposed
**Date**: $(date +%Y-%m-%d)
**Author**: [Your Name]
**Tags**: [tag1, tag2, tag3]

## Context
What situation are we addressing? What forces are at play?

## Decision
What are we doing? What specific choice are we making?

## Consequences
What becomes easier or more difficult to do because of this change?

## Alternatives Considered
What other options did we evaluate? Why did we reject them?

## References
Links to relevant discussions, RFCs, documentation
EOF
    fi

    echo "Created ADR: $adr_file"
    echo "Don't forget to update the ADR index!"
}

cleanup() {
    echo "Cleaning up .claude directory..."
    echo ""

    # Create docs/ subdirectory for non-ADR documents
    mkdir -p "$CLAUDE_DIR/docs"

    # Legacy/redundant files to remove or consolidate
    local legacy_files=(
        "ADR-SYSTEM-GUIDE.md"
        "README.md"
    )

    # Specialized docs to move to docs/ subdirectory
    local specialized_docs=(
        "RUST-PROJECT-SETUP.md"
        "context-management-prompt.md"
        "project-context.md"
    )

    echo "Moving specialized documents to docs/ subdirectory:"
    for doc in "${specialized_docs[@]}"; do
        if [[ -f "$CLAUDE_DIR/$doc" ]]; then
            mv "$CLAUDE_DIR/$doc" "$CLAUDE_DIR/docs/"
            echo "  ✓ Moved $doc to docs/"
        fi
    done

    echo ""
    echo "Legacy files found (will prompt for removal):"
    for file in "${legacy_files[@]}"; do
        if [[ -f "$CLAUDE_DIR/$file" ]]; then
            echo "  • $file"
            read -p "Remove $file? (y/N): " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                rm "$CLAUDE_DIR/$file"
                echo "    ✓ Removed"
            else
                echo "    - Kept"
            fi
        fi
    done

    echo ""
    echo "Cleanup complete!"
}

merge_adr() {
    local adr_path="$1"
    local merged_dir="$CLAUDE_DIR/merged"

    if [[ ! -f "$adr_path" ]]; then
        echo "ERROR: ADR file not found: $adr_path"
        exit 1
    fi

    mkdir -p "$merged_dir"

    local filename=$(basename "$adr_path")
    local merged_path="$merged_dir/$filename"

    if safe_git "merge ADR" "add_files" add "$adr_path"; then
        mv "$adr_path" "$merged_path"
        echo "ADR merged to: $merged_path"
        echo "Don't forget to update the ADR index status!"
    else
        echo "Failed to merge ADR"
        exit 1
    fi
}

show_help() {
    cat << 'EOF'
ADR Helper Script v2.0 - Claude Context System

Usage: ./adr-helper.sh [command] [options]

Commands:
  validate              Validate ADR structure and check for missing files
  list                  List all active ADRs
  status                Show ADR system status and statistics
  new <category> <title>  Create a new ADR in the specified category
  merge <adr-path>      Move an ADR to merged/ directory
  cleanup               Clean up legacy files and organize .claude directory
  help                  Show this help message

Categories:
  feat    - Feature decisions and implementations
  arch    - Architecture and design decisions
  docs    - Documentation decisions
  chore   - Process and tooling decisions

Examples:
  ./adr-helper.sh new feat "docker-client-manager-pattern"
  ./adr-helper.sh new arch "error-handling-strategy"
  ./adr-helper.sh merge .claude/branches/feat/docker-client.md
  ./adr-helper.sh status

Configuration:
  Edit .claude/adr-index.toml to configure permissions and team settings.

EOF
}

# Main command dispatcher
case "${1:-help}" in
    validate)
        validate
        ;;
    list)
        list
        ;;
    status)
        status
        ;;
    new)
        new_adr "$2" "$3"
        ;;
    merge)
        merge_adr "$2"
        ;;
    cleanup)
        cleanup
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "Unknown command: $1"
        echo "Run './adr-helper.sh help' for usage information"
        exit 1
        ;;
esac
