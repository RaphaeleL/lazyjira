#!/usr/bin/env bash

# =====================================================================
# LazyJira - Simple CLI and TUI for Jira Interactions 
# =====================================================================
#
# Features:
#
#   Create issues
#   Search issues via JQL (Jira Query Language)
#   Show own or other users' tickets
#   Transition tickets (start / done / todo)
#   Search by component
#   Interactive issue picker using fzf
#
# Usage Examples:
#
#   jira create --name "fix pipeline"
#   jira update start ITGADT-2836
#   jira update done ITGADT-2836
#   jira search 'project = ITGADT AND status != Done'
#   jira search 'assignee = currentUser() AND resolution = Unresolved ORDER BY updated DESC'
#   jira from xcxa1b9
#   jira from xcxa1b4,xcxa1b9
#   jira mine
#   jira pick
#   jira component "FSV/IS"
#
# ---------------------------------------------------------------------
# Author  : Raphaele Salvatore Licciardo
# Created : 14 Apr 2026
# Changed : 14 Apr 2026
# Version : 0.1.0
# ---------------------------------------------------------------------
#
# Requirements:
#   curl, jq, fzf, awk, tr, sort
#
# =====================================================================

set -euo pipefail

ENV_FILE="$HOME/.jira/env"
mkdir -p "$HOME/.jira"
if [[ -f "$ENV_FILE" ]]; then
    source "$ENV_FILE"
fi
if [[ -z "${TOKEN:-}" ]]; then
    echo "Jira token not found."

    printf "Enter your Jira Bearer Token: "
    read -r TOKEN

    if [[ -z "$TOKEN" ]]; then
        echo "❌ No token provided. Aborting."
        exit 1
    fi

    umask 077
    printf "TOKEN=%s\n" "$TOKEN" > "$ENV_FILE"
    echo "Token saved to $ENV_FILE"
fi

JIRA_URL="https://jira.rz.bankenit.de/jira"
AUTH_HEADER="Authorization: Bearer $TOKEN"

DEFAULT_PROJECT="ITGADT"
CACHE_DIR="$HOME/.jira/cache"
CACHE_FILE="$CACHE_DIR/issues.json"

mkdir -p "$CACHE_DIR" "$HOME/.jira"

get_status_names() {
    case "$1" in
        done)
            printf "%s\n" "Done" "Fertig" "Abgeschlossen"
            ;;
        start)
            printf "%s\n" "In Arbeit" "In Progress" "Started"
            ;;
        todo)
            printf "%s\n" "Zu erledigen" "To Do" "Open"
            ;;
        *)
            return 1
            ;;
    esac
}

api() {
    curl -s -H "$AUTH_HEADER" -H "Content-Type: application/json" "$@"
}

cache_issues() {
    local jql='assignee = currentUser() ORDER BY updated DESC'
    local tmp

    tmp="$(api "$JIRA_URL/rest/api/2/search?jql=$(printf '%s' "$jql" | jq -sRr @uri)")"

    echo "$tmp" | jq -e '.issues' >/dev/null 2>&1 || {
        echo "❌ invalid Jira response"
        echo "$tmp" | head -c 300
        return 1
    }

    echo "$tmp" > "$CACHE_FILE"
}

issues() {
    [[ -f "$CACHE_FILE" ]] || return 0

    jq -r '
        .issues // [] | .[] |
        "\(.key) \(.fields.status.name) \(.fields.summary)"
    ' "$CACHE_FILE" 2>/dev/null || true
}

pick() {
    cache_issues >/dev/null 2>&1 || true
    issues | fzf --prompt="jira> " | awk '{print $1}'
}

component_search() {
    local comp="${1:-}"

    [[ -z "$comp" ]] && {
        echo "usage: jira component <name>"
        exit 1
    }

    local norm
    norm="$(echo "$comp" | awk -F/ '{ for (i=1;i<=NF;i++) $i=toupper($i) print }' OFS="/")"

    local jql="component = \"${norm}\" ORDER BY updated DESC"

    api "$JIRA_URL/rest/api/2/search?jql=$(printf '%s' "$jql" | jq -sRr @uri)" |
        jq -r '.issues // [] | .[] | "\(.key) [\(.fields.status.name)] \(.fields.summary)"'
}

require_tools() {
    local missing=0
    local tools=(curl jq fzf awk tr sort)
    local is_macos=0

    [[ "$(uname)" == "Darwin" ]] && is_macos=1

    for cmd in "${tools[@]}"; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            if [[ $missing -eq 0 ]]; then
                echo "❌ Missing tools:"
            fi

            echo " - $cmd"

            if [[ $is_macos -eq 1 ]]; then
                case "$cmd" in
                    jq|fzf)
                        echo " -> brew install $cmd"
                        ;;
                    curl|awk|tr|sort)
                        echo " -> should be provided by macOS (check PATH)"
                        ;;
                    *)
                        echo " -> brew install $cmd"
                        ;;
                esac
            fi

            missing=1
        fi
    done

    if [[ $missing -eq 1 ]]; then
        echo ""
        [[ $is_macos -eq 1 ]] && echo "Install missing tools via Homebrew."
        exit 1
    else
        echo "All required tools installed"
    fi
}

resolve_transition_id() {
    local issue="$1"
    local target="$2"

    local names
    names="$(get_status_names "$target")" || {
        echo "Unknown state: $target"
        return 1
    }

    api "$JIRA_URL/rest/api/2/issue/$issue/transitions" |
        jq -r --arg names "$names" '
            ($names | split("\n")) as $list
            | .transitions // []
            | .[]
            | select(.name as $n | $list | index($n))
            | .id
        ' | head -n 1
}

transition() {
    local target="$1"
    local issue="${2:-}"

    if [[ -z "$issue" ]]; then
        issue="$(pick || true)"
    fi

    [[ -z "$issue" ]] && {
        echo "No issue selected"
        exit 1
    }

    local id
    id=$(resolve_transition_id "$issue" "$target")

    if [[ -z "$id" ]]; then
        echo "No transition found for '$target' on $issue"
        exit 1
    fi

    api -X POST "$JIRA_URL/rest/api/2/issue/$issue/transitions" \
        -d "{\"transition\": {\"id\": \"$id\"}}" >/dev/null

    echo "Updated Ticket $issue to $target"

    cache_issues >/dev/null 2>&1 || true
}

create() {
    local project=""
    local component=""
    local name=""
    local desc=""
    local issuetype="10100"

    while [[ $# -gt 0 ]]; do
        case "$1" in
            --project) project="$2"; shift 2 ;;
            --component) component="$2"; shift 2 ;;
            --name) name="$2"; shift 2 ;;
            --desc) desc="$2"; shift 2 ;;
            --type) issuetype="$2"; shift 2 ;;
            *)
                echo "unknown arg: $1"
                exit 1
                ;;
        esac
    done

    project="${project:-$DEFAULT_PROJECT}"

    [[ -z "$name" ]] && {
        echo "usage: jira create --name X [--project KEY] [--desc X]"
        exit 1
    }

    payload=$(jq -n \
        --arg p "$project" \
        --arg n "$name" \
        --arg d "$desc" \
        --arg t "$issuetype" \
        --arg c "$component" '
        {
            fields: {
                project: { key: $p },
                summary: $n,
                description: $d,
                issuetype: { id: $t }
            }
            | if $c != "" then .components = [{ name: $c }] else . end
        }'
    )

    api -X POST "$JIRA_URL/rest/api/2/issue" -d "$payload" | jq

    cache_issues >/dev/null 2>&1 || true
}

search() {
    local jql="${*:-assignee = currentUser()}"

    api "$JIRA_URL/rest/api/2/search?jql=$(printf '%s' "$jql" | jq -sRr @uri)" |
        jq -r '.issues // [] | .[] | "\(.key) [\(.fields.status.name)] \(.fields.summary)"'
}

from_users() {
    local users="${1:-}"
    local jql="assignee in ($users) ORDER BY updated DESC"
    search "$jql"
}

mine() {
    jq -r '
        .issues // [] | .[] |
        "\(.key) [\(.fields.status.name)] \(.fields.summary)"
    ' "$CACHE_FILE"
}

doctor() {
    if [[ -f "$HOME/.jira/env" ]]; then
        echo "Token: found"
    else
        echo "Token: not found. Create one in your Jira"
    fi

    if [[ -f "$CACHE_FILE" ]]; then
        printf "Cache items: "
        jq '.issues | length' "$CACHE_FILE" 2>/dev/null || echo "broken cache"
    else
        echo "no cache"
    fi

    printf "Checking dependencies: "
    require_tools || echo true
}

help() {
    if [ -t 1 ]; then
        BOLD=$'\033[1m'
        RESET=$'\033[0m'
    else
        BOLD=""
        RESET=""
    fi

    cat <<EOF
${BOLD}LazyJira CLI - Simple CLI for Jira Interactions${RESET}

${BOLD}Usage:${RESET} jira [command] [options]

${BOLD}Core Commands:${RESET}
  mine
  from userA[,userB,...]
  search '<JQL>'
  component <name>
  pick
  update <done|start|todo> [KEY]
  doctor
  create --name "title" [--project KEY] [--desc TEXT]

${BOLD}Examples:${RESET}
  jira create --name "fix pipeline"
  jira update start ITGADT-2836
  jira update done ITGADT-2836
  jira search 'project = ITGADT AND status != Done'
  jira mine
  jira pick
  jira component "FSV/IS"
EOF
}

cmd="${1:-}"
shift || true

case "$cmd" in
    mine) cache_issues; mine ;;
    from) from_users "$@" ;;
    pick) pick ;;
    search) search "$@" ;;
    create) create "$@" ;;
    component) component_search "$@" ;;
    doctor) doctor ;;
    help|"") help ;;
    update)
        state="${1:-}"
        key="${2:-}"

        case "$state" in
            done|start|todo) ;;
            *)
                echo "invalid state: $state"
                exit 1
                ;;
        esac

        transition "$state" "$key"
        ;;
    *)
        help
        ;;
esac
