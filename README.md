# LazyJira

Simple CLI and TUI tools for Jira interactions. Manage your Jira tickets efficiently from the command line or through an interactive terminal interface.

## Features

> **This is a work-in-progress project currently under active development.**  

Features may be incomplete, unstable, or subject to change. Please use with caution and expect potential issues. This software comes with no warranties.

### CLI Tool (`jira`)
- **View Issues**: Show your assigned issues, issues by user, or search with JQL
- **Interactive Selection**: Use `fzf` for interactive issue picking
- **Create Issues**: Create new tickets with custom fields
- **Update Status**: Transition tickets between states (todo/start/done/review/waiting)
- **Show Details**: Display comprehensive ticket information
- **Component Search**: Find issues by component
- **Caching**: Automatic caching of issue data for faster access

### TUI Tool (`lazyjira`)
- **Interactive Interface**: Browse and manage issues in a terminal UI
- **Real-time Search**: Search issues with JQL queries
- **Issue Details**: View full ticket information with markdown rendering
- **Keyboard Navigation**: Vim-like navigation and shortcuts

## Installation

### Prerequisites

- Rust (latest stable)
- `fzf` (for CLI interactive selection)

### Build and Install

```bash
# Clone the repository
git clone https://github.com/RaphaeleL/lazyjira
cd lazyjira

# Build both tools
make build

# Install to ~/.local/bin
make install

# Or create symlinks for development
make link
```

This will install:
- `jira` - The CLI tool
- `lazyjira` - The TUI tool

Make sure `~/.local/bin` is in your PATH.

## Configuration

Both tools share the same configuration file located at `~/Library/Application Support/jira.lazyjira.LazyJira/env`.

### Initial Setup

Once you run either `jira` or `lazyjira`, it will automatically create the config and prompt you for:

- Jira Bearer Token
- Jira URL
- Default Project
- Issue Type ID

The config file contains thereby:

```bash
TOKEN=your_jira_token
JIRA_URL=https://your-jira-instance.com
DEFAULT_PROJECT=YOUR_PROJECT
ISSUE_TYPE=your_issue_type_id
```

## CLI Usage

### Basic Commands

```bash
# Show your assigned issues
jira mine

# Show issues assigned to specific users
jira from username1,username2

# Interactive issue picker (requires fzf)
jira pick

# Search with JQL
jira search 'project = MYPROJECT AND status != Done'

# Show detailed information about a ticket
jira show TICKET-123

# Create a new issue
jira create --name "Fix the bug" --desc "Detailed description"

# Update ticket status
jira update done TICKET-123
jira update start TICKET-123
jira update todo TICKET-123

# Search by component
jira component "Component Name"

# Check configuration and cache status
jira doctor
```

### Available Status Transitions

- `todo` --> "Zu erledigen" (To Do)
- `start` --> "Wird Ausgeführt" (In Progress)
- `done` --> "Fertig" (Done)
- `review` --> "In Review"
- `waiting` --> "Waiting"

### Advanced Examples

```bash
# Create issue with all options
jira create \
  --name "Implement new feature" \
  --project MYPROJECT \
  --component "Backend" \
  --desc "This feature will..." \
  --type "Task"

# Complex JQL search
jira search 'assignee = currentUser() AND updated > -1w'

# Show issues from multiple users
jira from 'user1,user2,user3'
```

## TUI Usage

Launch the interactive terminal interface:
```bash
lazyjira
```

### Navigation

- `j/k` - Navigate issues
- `/` - Update the currentJQL 
- `q` - Quit

### Features

- **Top Panel**: Short Information of the Current Ticket
- **Left Panel**: Issue list with status indicators
- **Right Panel**: Detailed issue information
- **Bottom Panel**: Current JQL query and status

## Troubleshooting

The CLI Command `jira doctor` checks configuration and dependencies.

### Jira Permissions

Your Jira token needs appropriate permissions for:

- Reading issues (`read:jira-work`)
- Creating issues (`write:jira-work`)
- Transitioning issues (`write:jira-work`)

### Config file not found

```bash
jira init
```

### No transition found

The CLI will show available transitions. Use the appropriate command:

```bash
jira update start TICKET-123
```

### fzf not found

Install fzf for interactive selection:

```bash
# macOS
brew install fzf

# Ubuntu/Debian
apt install fzf
```
