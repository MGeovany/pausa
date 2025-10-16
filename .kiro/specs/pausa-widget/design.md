# Design Document

## Overview

Pausa is a desktop productivity widget built with Tauri (Rust backend) and React (TypeScript frontend) that provides focus session management with a Raycast-inspired interface. The application consists of multiple window types: a command palette overlay, a floating focus widget, and fullscreen break overlays. The system uses a state machine architecture to manage focus/break cycles and integrates with OS-level APIs for application/website blocking.

## Architecture

### High-Level Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   React Frontend │    │  Tauri Commands │    │   Rust Backend  │
│                 │    │                 │    │                 │
│ • Command Palette│◄──►│ • Window Mgmt   │◄──►│ • State Machine │
│ • Focus Widget   │    │ • Global Hotkeys│    │ • App Blocking  │
│ • Break Overlay  │    │ • System Tray   │    │ • Data Storage  │
│ • Settings UI    │    │ • Notifications │    │ • Timer Service │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 │
                    ┌─────────────────┐
                    │   OS Integration │
                    │                 │
                    │ • Global Hotkeys │
                    │ • Process Monitor│
                    │ • Hosts File     │
                    │ • Notifications  │
                    └─────────────────┘
```

### Window Management Strategy

The application uses multiple Tauri windows with different configurations:

1. **Command Palette Window**: Modal overlay, always-on-top, transparent background
2. **Focus Widget Window**: Small floating window, draggable, always-on-top, click-through when inactive
3. **Break Overlay Window**: Fullscreen, always-on-top, blocks input in strict mode
4. **Settings Window**: Standard modal window

## Components and Interfaces

### Frontend Components

#### 1. Command Palette (`CommandPalette.tsx`)
```typescript
interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  commands: Command[];
}

interface Command {
  id: string;
  label: string;
  category: 'focus' | 'break' | 'lock' | 'stats' | 'settings';
  shortcut?: string;
  action: () => Promise<void>;
}
```

**Responsibilities:**
- Render search input with real-time filtering
- Display categorized command list with keyboard navigation
- Handle command execution and window closing
- Show keyboard shortcuts and hints

#### 2. Focus Widget (`FocusWidget.tsx`)
```typescript
interface FocusWidgetProps {
  session: FocusSession | null;
  onToggleSession: () => void;
  onResetSession: () => void;
  onOpenMenu: () => void;
}

interface FocusSession {
  id: string;
  startTime: Date;
  duration: number; // seconds
  remaining: number; // seconds
  isRunning: boolean;
  isStrict: boolean;
  state: 'idle' | 'running' | 'pre-alert' | 'ending';
}
```

**Responsibilities:**
- Display circular progress indicator with remaining time
- Provide session control buttons (play/pause, reset, menu)
- Handle drag-and-drop repositioning
- Show pre-alert visual feedback
- Display strict mode indicator

#### 3. Break Overlay (`BreakOverlay.tsx`)
```typescript
interface BreakOverlayProps {
  breakSession: BreakSession;
  onCompleteBreak: () => void;
  onEmergencyOverride: (pin: string) => Promise<boolean>;
}

interface BreakSession {
  id: string;
  type: 'short' | 'long';
  duration: number;
  remaining: number;
  activity: BreakActivity;
  allowEmergency: boolean;
}

interface BreakActivity {
  title: string;
  description: string;
  checklist: string[];
}
```

**Responsibilities:**
- Display fullscreen break interface with countdown
- Show suggested break activity with checklist
- Handle emergency PIN override
- Prevent input when in strict mode

#### 4. Settings Panel (`Settings.tsx`)
```typescript
interface SettingsProps {
  settings: UserSettings;
  onUpdateSettings: (settings: Partial<UserSettings>) => Promise<void>;
}

interface UserSettings {
  focusDuration: number; // minutes
  shortBreakDuration: number; // minutes
  longBreakDuration: number; // minutes
  cyclesPerLongBreak: number;
  preAlertSeconds: number;
  strictMode: boolean;
  pinHash?: string;
  blockedApps: string[];
  blockedWebsites: string[];
}
```

### Backend Services

#### 1. State Manager (`state_manager.rs`)
```rust
pub struct StateManager {
    current_state: AppState,
    session_data: Option<SessionData>,
    settings: UserSettings,
    timer: Option<Timer>,
}

pub enum AppState {
    Idle,
    FocusRunning,
    FocusPreAlert,
    FocusEnding,
    BreakRunning,
    LongBreakRunning,
}

impl StateManager {
    pub fn start_focus_session(&mut self, strict: bool) -> Result<()>;
    pub fn pause_session(&mut self) -> Result<()>;
    pub fn end_session(&mut self) -> Result<()>;
    pub fn start_break(&mut self, break_type: BreakType) -> Result<()>;
    pub fn handle_timer_tick(&mut self) -> Result<Vec<StateEvent>>;
}
```

#### 2. Blocking Service (`blocking_service.rs`)
```rust
pub struct BlockingService {
    blocked_apps: Vec<String>,
    blocked_websites: Vec<String>,
    is_active: bool,
}

impl BlockingService {
    pub fn enable_blocking(&mut self) -> Result<()>;
    pub fn disable_blocking(&mut self) -> Result<()>;
    pub fn add_blocked_app(&mut self, app_name: String) -> Result<()>;
    pub fn add_blocked_website(&mut self, domain: String) -> Result<()>;
    pub fn check_and_block_processes(&self) -> Result<Vec<String>>;
}
```

#### 3. Window Manager (`window_manager.rs`)
```rust
pub struct WindowManager {
    app_handle: AppHandle,
    windows: HashMap<WindowType, Window>,
}

pub enum WindowType {
    CommandPalette,
    FocusWidget,
    BreakOverlay,
    Settings,
}

impl WindowManager {
    pub fn show_command_palette(&self) -> Result<()>;
    pub fn show_focus_widget(&self) -> Result<()>;
    pub fn show_break_overlay(&self, monitor: Monitor) -> Result<()>;
    pub fn hide_window(&self, window_type: WindowType) -> Result<()>;
}
```

### Tauri Commands Interface

```rust
#[tauri::command]
async fn start_focus_session(strict: bool, state: State<'_, Mutex<StateManager>>) -> Result<FocusSession>;

#[tauri::command]
async fn pause_session(state: State<'_, Mutex<StateManager>>) -> Result<()>;

#[tauri::command]
async fn get_current_session(state: State<'_, Mutex<StateManager>>) -> Result<Option<FocusSession>>;

#[tauri::command]
async fn update_settings(settings: UserSettings, state: State<'_, Mutex<StateManager>>) -> Result<()>;

#[tauri::command]
async fn get_session_stats(days: u32) -> Result<Vec<SessionStats>>;

#[tauri::command]
async fn verify_emergency_pin(pin: String, state: State<'_, Mutex<StateManager>>) -> Result<bool>;
```

## Data Models

### Database Schema (SQLite)

```sql
-- User configuration
CREATE TABLE user_settings (
    id INTEGER PRIMARY KEY,
    focus_duration INTEGER NOT NULL DEFAULT 1500, -- 25 minutes in seconds
    short_break_duration INTEGER NOT NULL DEFAULT 300, -- 5 minutes
    long_break_duration INTEGER NOT NULL DEFAULT 900, -- 15 minutes
    cycles_per_long_break INTEGER NOT NULL DEFAULT 4,
    pre_alert_seconds INTEGER NOT NULL DEFAULT 120,
    strict_mode BOOLEAN NOT NULL DEFAULT FALSE,
    pin_hash TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Blocked applications and websites
CREATE TABLE block_list (
    id INTEGER PRIMARY KEY,
    type TEXT NOT NULL CHECK (type IN ('app', 'website')),
    value TEXT NOT NULL,
    platform TEXT, -- 'windows', 'macos', 'linux', or NULL for all
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Focus and break sessions
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    session_type TEXT NOT NULL CHECK (session_type IN ('focus', 'short_break', 'long_break')),
    start_time DATETIME NOT NULL,
    end_time DATETIME,
    planned_duration INTEGER NOT NULL, -- seconds
    actual_duration INTEGER, -- seconds, NULL if not completed
    strict_mode BOOLEAN NOT NULL DEFAULT FALSE,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    notes TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Evasion attempts (when user tries to access blocked content)
CREATE TABLE evasion_attempts (
    id INTEGER PRIMARY KEY,
    session_id TEXT NOT NULL,
    attempt_type TEXT NOT NULL CHECK (attempt_type IN ('app', 'website')),
    blocked_item TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions (id)
);

-- Computed insights and statistics
CREATE TABLE insights (
    id INTEGER PRIMARY KEY,
    metric_key TEXT NOT NULL,
    metric_value REAL NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,
    computed_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for performance
CREATE INDEX idx_sessions_start_time ON sessions (start_time);
CREATE INDEX idx_sessions_type ON sessions (session_type);
CREATE INDEX idx_block_list_type_value ON block_list (type, value);
CREATE INDEX idx_evasion_attempts_session ON evasion_attempts (session_id);
CREATE INDEX idx_insights_key_period ON insights (metric_key, period_start, period_end);
```

### Rust Data Structures

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct UserSettings {
    pub focus_duration: u32,
    pub short_break_duration: u32,
    pub long_break_duration: u32,
    pub cycles_per_long_break: u32,
    pub pre_alert_seconds: u32,
    pub strict_mode: bool,
    pub pin_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FocusSession {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub duration: u32,
    pub remaining: u32,
    pub is_running: bool,
    pub is_strict: bool,
    pub state: SessionState,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SessionState {
    Idle,
    Running,
    PreAlert,
    Ending,
}

#[derive(Serialize, Deserialize)]
pub struct SessionStats {
    pub date: String,
    pub focus_minutes: u32,
    pub breaks_completed: u32,
    pub sessions_completed: u32,
    pub evasion_attempts: u32,
}
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum PausaError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("Window management error: {0}")]
    Window(String),
    
    #[error("Blocking service error: {0}")]
    Blocking(String),
    
    #[error("Invalid session state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: AppState, to: AppState },
    
    #[error("Authentication failed")]
    AuthenticationFailed,
    
    #[error("System integration error: {0}")]
    SystemIntegration(String),
}

pub type Result<T> = std::result::Result<T, PausaError>;
```

### Error Handling Strategy

1. **Frontend Error Handling**: Use React error boundaries and toast notifications for user-facing errors
2. **Backend Error Handling**: Log errors and return structured error responses to frontend
3. **System Integration Errors**: Graceful degradation when OS-level features are unavailable
4. **Database Errors**: Automatic retry for transient errors, user notification for persistent issues


## Platform-Specific Considerations

### macOS
- Use `NSWorkspace` for application monitoring
- Implement global hotkeys with `CGEventTap`
- Use Accessibility APIs for window management
- Modify `/etc/hosts` for website blocking (requires admin privileges)

### Windows
- Use WMI for process monitoring
- Implement global hotkeys with `RegisterHotKey`
- Use Windows API for window management
- Modify hosts file for website blocking

### Linux
- Use `wmctrl` and `xdotool` for window management
- Monitor processes via `/proc` filesystem
- Implement hotkeys with X11 or Wayland protocols
- Modify `/etc/hosts` for website blocking

## Security Considerations

1. **PIN Storage**: Use bcrypt or similar for hashing emergency PINs
2. **Process Monitoring**: Implement safeguards against process injection
3. **File System Access**: Validate all file paths and use appropriate permissions
4. **Network Blocking**: Implement DNS-level blocking as backup to hosts file
5. **Data Encryption**: Encrypt sensitive user data in the database
