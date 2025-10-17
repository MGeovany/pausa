# Design Document

## Overview

The Pausa onboarding and work cycle system extends the existing Pausa application with a guided setup wizard and automated productivity cycle management. Built on the existing Tauri + React architecture, this system introduces new window types for onboarding flows, enhanced state management for work cycles, and integration with the existing focus widget and break overlay systems.

## Architecture

### High-Level Architecture Extension

```
┌─────────────────────────────────────────────────────────────────┐
│                    Existing Pausa Architecture                  │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   React Frontend│  Tauri Commands │      Rust Backend           │
│                 │                 │                             │
│ • Command Palette│ • Window Mgmt   │ • State Machine            │
│ • Focus Widget   │ • Global Hotkeys│ • App Blocking             │
│ • Break Overlay  │ • System Tray   │ • Data Storage             │
│ • Settings UI    │ • Notifications │ • Timer Service            │
└─────────────────┴─────────────────┴─────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────────┐
│                    New Onboarding & Cycle System                │
├─────────────────┬─────────────────┬─────────────────────────────┤
│   React Frontend│  Tauri Commands │      Rust Backend           │
│                 │                 │                             │
│ • Welcome Screen │ • Onboarding    │ • Onboarding State Manager │
│ • Setup Wizard   │   Commands      │ • Work Schedule Service    │
│ • Cycle UI       │ • Cycle Mgmt    │ • Cycle Orchestrator       │
│ • Notifications  │ • Notifications │ • Enhanced Timer Service   │
└─────────────────┴─────────────────┴─────────────────────────────┘
```

### State Flow Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  First Launch   │───►│  Onboarding     │───►│   Main App      │
│                 │    │   Wizard        │    │   (Existing)    │
│ • Welcome       │    │                 │    │                 │
│ • Detection     │    │ • Work Schedule │    │ • Command Palette│
└─────────────────┘    │ • Cycle Config  │    │ • Focus Widget  │
                       │ • Strict Mode   │    │ • Break Overlay │
                       │ • Summary       │    │ • Settings      │
                       └─────────────────┘    └─────────────────┘
                                │                       │
                                └───────────────────────┘
                                          │
                               ┌─────────────────┐
                               │  Work Cycle     │
                               │  Orchestrator   │
                               │                 │
                               │ • Focus Timer   │
                               │ • Break Timer   │
                               │ • Notifications │
                               │ • State Mgmt    │
                               └─────────────────┘
```

## Components and Interfaces

### Frontend Components

#### 1. Onboarding Wizard (`OnboardingWizard.tsx`)

```typescript
interface OnboardingWizardProps {
  onComplete: (config: OnboardingConfig) => Promise<void>;
  onSkip?: () => void;
}

interface OnboardingConfig {
  useWorkSchedule: boolean;
  workHours?: {
    start: string; // "09:00"
    end: string;   // "18:00"
  };
  focusDuration: number; // minutes
  breakDuration: number; // minutes
  longBreakDuration: number; // minutes
  cyclesPerLongBreak: number;
  strictMode: boolean;
  emergencyKey?: string; // key combination
}

interface OnboardingStep {
  id: string;
  title: string;
  subtitle?: string;
  component: React.ComponentType<StepProps>;
  validation?: (data: any) => boolean;
}
```

**Step Components:**
- `WelcomeStep`: Simple welcome screen with logo and description
- `WorkScheduleStep`: Choice between work schedule or manual configuration
- `WorkHoursStep`: Time picker for work hours
- `CycleConfigStep`: Duration selectors for focus/break cycles
- `StrictModeStep`: Strict mode toggle and emergency key setup
- `SummaryStep`: Configuration review and confirmation

#### 2. Work Cycle Manager (`WorkCycleManager.tsx`)

```typescript
interface WorkCycleManagerProps {
  config: WorkCycleConfig;
  onStateChange: (state: CycleState) => void;
}

interface WorkCycleConfig {
  focusDuration: number;
  breakDuration: number;
  longBreakDuration: number;
  cyclesPerLongBreak: number;
  strictMode: boolean;
  workHours?: TimeRange;
  emergencyKey?: string;
}

interface CycleState {
  phase: 'idle' | 'focus' | 'break' | 'long_break';
  remaining: number; // seconds
  cycleCount: number;
  isRunning: boolean;
  canStart: boolean; // based on work hours
}
```

#### 3. Cycle Notifications (`CycleNotifications.tsx`)

```typescript
interface CycleNotificationsProps {
  state: CycleState;
  config: WorkCycleConfig;
  userName?: string;
}

interface NotificationMessage {
  id: string;
  type: 'focus_start' | 'focus_warning' | 'break_start' | 'break_end';
  title: string;
  message: string;
  emoji?: string;
  duration?: number; // ms
}
```

### Backend Services

#### 1. Onboarding State Manager (`onboarding_manager.rs`)

```rust
pub struct OnboardingManager {
    current_step: OnboardingStep,
    collected_data: HashMap<String, serde_json::Value>,
    is_complete: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnboardingStep {
    Welcome,
    WorkSchedule,
    WorkHours,
    CycleConfig,
    StrictMode,
    Summary,
    Complete,
}

impl OnboardingManager {
    pub fn new() -> Self;
    pub fn next_step(&mut self) -> Result<OnboardingStep>;
    pub fn previous_step(&mut self) -> Result<OnboardingStep>;
    pub fn set_step_data(&mut self, step: OnboardingStep, data: serde_json::Value) -> Result<()>;
    pub fn complete_onboarding(&mut self) -> Result<WorkCycleConfig>;
    pub fn skip_to_step(&mut self, step: OnboardingStep) -> Result<()>;
}
```

#### 2. Work Cycle Orchestrator (`cycle_orchestrator.rs`)

```rust
pub struct CycleOrchestrator {
    config: WorkCycleConfig,
    current_state: CycleState,
    timer: Timer,
    notification_service: NotificationService,
    cycle_counter: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CyclePhase {
    Idle,
    Focus,
    Break,
    LongBreak,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleState {
    phase: CyclePhase,
    remaining_seconds: u32,
    cycle_count: u32,
    is_running: bool,
    can_start: bool,
}

impl CycleOrchestrator {
    pub fn new(config: WorkCycleConfig) -> Self;
    pub fn start_focus_session(&mut self) -> Result<()>;
    pub fn start_break(&mut self, is_long: bool) -> Result<()>;
    pub fn pause_current(&mut self) -> Result<()>;
    pub fn resume_current(&mut self) -> Result<()>;
    pub fn end_session(&mut self) -> Result<()>;
    pub fn get_current_state(&self) -> CycleState;
    pub fn is_within_work_hours(&self) -> bool;
    pub fn handle_timer_tick(&mut self) -> Result<Vec<CycleEvent>>;
}
```

#### 3. Enhanced Notification Service (`notification_service.rs`)

```rust
pub struct NotificationService {
    user_name: Option<String>,
    sound_enabled: bool,
    message_templates: HashMap<NotificationType, MessageTemplate>,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    FocusStart,
    FocusWarning,
    FocusEnd,
    BreakStart,
    BreakEnd,
    LongBreakStart,
    CycleComplete,
}

#[derive(Debug, Clone)]
pub struct MessageTemplate {
    title: String,
    message: String,
    emoji: Option<String>,
    sound: Option<String>,
}

impl NotificationService {
    pub fn new() -> Self;
    pub fn send_notification(&self, notification_type: NotificationType) -> Result<()>;
    pub fn send_custom_notification(&self, title: &str, message: &str) -> Result<()>;
    pub fn set_user_name(&mut self, name: String);
    pub fn enable_sounds(&mut self, enabled: bool);
}
```

### Tauri Commands Interface

```rust
// Onboarding Commands
#[tauri::command]
async fn start_onboarding() -> Result<OnboardingStep>;

#[tauri::command]
async fn next_onboarding_step(
    step_data: serde_json::Value,
    state: State<'_, Mutex<OnboardingManager>>
) -> Result<OnboardingStep>;

#[tauri::command]
async fn complete_onboarding(
    final_config: OnboardingConfig,
    state: State<'_, Mutex<OnboardingManager>>
) -> Result<()>;

// Work Cycle Commands
#[tauri::command]
async fn start_work_cycle(
    state: State<'_, Mutex<CycleOrchestrator>>
) -> Result<CycleState>;

#[tauri::command]
async fn pause_work_cycle(
    state: State<'_, Mutex<CycleOrchestrator>>
) -> Result<CycleState>;

#[tauri::command]
async fn end_work_session(
    state: State<'_, Mutex<CycleOrchestrator>>
) -> Result<()>;

#[tauri::command]
async fn get_cycle_state(
    state: State<'_, Mutex<CycleOrchestrator>>
) -> Result<CycleState>;
```

## Data Models

### Database Schema Extensions

```sql
-- Onboarding completion tracking
CREATE TABLE onboarding_completion (
    id INTEGER PRIMARY KEY,
    completed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    version TEXT NOT NULL,
    config_snapshot TEXT -- JSON of final configuration
);

-- Work schedule configuration
CREATE TABLE work_schedule (
    id INTEGER PRIMARY KEY,
    user_id INTEGER DEFAULT 1,
    use_work_schedule BOOLEAN NOT NULL DEFAULT FALSE,
    work_start_time TEXT, -- "09:00"
    work_end_time TEXT,   -- "18:00"
    timezone TEXT DEFAULT 'local',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Cycle configuration (extends existing user_settings)
ALTER TABLE user_settings ADD COLUMN cycles_per_long_break INTEGER DEFAULT 4;
ALTER TABLE user_settings ADD COLUMN emergency_key_combination TEXT;
ALTER TABLE user_settings ADD COLUMN user_name TEXT;

-- Work cycle sessions (extends existing sessions table)
ALTER TABLE sessions ADD COLUMN cycle_number INTEGER;
ALTER TABLE sessions ADD COLUMN is_long_break BOOLEAN DEFAULT FALSE;
ALTER TABLE sessions ADD COLUMN within_work_hours BOOLEAN DEFAULT TRUE;

-- Notification history
CREATE TABLE notification_history (
    id INTEGER PRIMARY KEY,
    session_id TEXT,
    notification_type TEXT NOT NULL,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    sent_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (session_id) REFERENCES sessions (id)
);
```

### Rust Data Structures

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingConfig {
    pub use_work_schedule: bool,
    pub work_hours: Option<WorkHours>,
    pub focus_duration: u32, // minutes
    pub break_duration: u32, // minutes
    pub long_break_duration: u32, // minutes
    pub cycles_per_long_break: u32,
    pub strict_mode: bool,
    pub emergency_key: Option<String>,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkHours {
    pub start: String, // "09:00"
    pub end: String,   // "18:00"
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkCycleConfig {
    pub focus_duration: u32,
    pub break_duration: u32,
    pub long_break_duration: u32,
    pub cycles_per_long_break: u32,
    pub strict_mode: bool,
    pub work_hours: Option<WorkHours>,
    pub emergency_key: Option<String>,
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleSession {
    pub id: String,
    pub phase: CyclePhase,
    pub start_time: DateTime<Utc>,
    pub planned_duration: u32, // seconds
    pub remaining: u32, // seconds
    pub cycle_number: u32,
    pub is_long_break: bool,
    pub within_work_hours: bool,
}
```

## Error Handling

### Error Types Extension

```rust
#[derive(Debug, thiserror::Error)]
pub enum OnboardingError {
    #[error("Invalid onboarding step transition: {from:?} -> {to:?}")]
    InvalidStepTransition { from: OnboardingStep, to: OnboardingStep },
    
    #[error("Missing required data for step: {step:?}")]
    MissingStepData { step: OnboardingStep },
    
    #[error("Invalid configuration: {reason}")]
    InvalidConfiguration { reason: String },
    
    #[error("Onboarding already completed")]
    AlreadyCompleted,
}

#[derive(Debug, thiserror::Error)]
pub enum CycleError {
    #[error("Cannot start cycle outside work hours")]
    OutsideWorkHours,
    
    #[error("Invalid cycle state transition: {from:?} -> {to:?}")]
    InvalidCycleTransition { from: CyclePhase, to: CyclePhase },
    
    #[error("Emergency key combination not set")]
    NoEmergencyKey,
    
    #[error("Notification service error: {0}")]
    NotificationError(String),
}
```

## User Experience Flow

### Onboarding Flow

```
1. Welcome Screen
   ├─ Logo + "Pausa"
   ├─ "A new way to work focused, without losing balance"
   └─ [Start Setup] → Step 2

2. Work Schedule Choice
   ├─ "Do you want Pausa to organize blocks according to work schedule?"
   ├─ [Yes, use my work schedule] → Step 3
   └─ [No, configure manually] → Step 4

3. Work Hours Definition (if Yes selected)
   ├─ "What hours do you normally work?"
   ├─ Start: [09:00 AM] End: [06:00 PM]
   └─ [Continue] → Step 4

4. Cycle Configuration
   ├─ Focus: [25 min] [30 min] [45 min] [Custom]
   ├─ Break: [5 min] [10 min] [15 min]
   ├─ Long break cycles: [3] [4]
   ├─ Long break duration: [15 min] [20 min]
   ├─ Recommendation: "We recommend 25 min focus..."
   └─ [Continue] → Step 5

5. Strict Mode Setup
   ├─ [✓] Enable strict mode (block screen during focus)
   ├─ Emergency key: [Press combination]
   ├─ Example: ⌘ + ⇧ + P
   └─ [Continue] → Step 6

6. Summary & Confirmation
   ├─ "All set"
   ├─ "Perfect, your routine is configured"
   ├─ "25 min focus / 5 min break / 4 cycles / 15 min long break"
   └─ [Start Pausa] → Main App
```

### Work Cycle Flow

```
Focus Session:
├─ Start → Minimize to tray
├─ System notification: "🧠 Focus mode started"
├─ Click tray → Show widget with time + controls
├─ 2 min warning → "⏳ Time to focus on wrapping up"
└─ End → Transition to break

Break Transition:
├─ "✨ Great work, [name]. Time to move around"
├─ "Active break in 2 minutes"
├─ App comes to foreground
└─ Show break interface

Break Session:
├─ "☕ Active break 05:00"
├─ "Move. Stretch. Drink water."
├─ Countdown timer
├─ (If strict mode) → Fullscreen overlay
└─ End → "Ready. Shall we start another block?"

Break Completion:
├─ [Start new block] → New focus session
└─ [End day session] → Return to idle

Long Break (every N cycles):
├─ "Excellent progress. Take a long break of X minutes"
├─ Same interface but larger timer
├─ Warmer color tone
└─ Reset cycle counter
```

## Integration with Existing System

### Window Management Integration

The onboarding system will use the existing window management infrastructure:

- **Onboarding Window**: New window type similar to Settings window
- **Enhanced Focus Widget**: Extend existing widget with cycle information
- **Enhanced Break Overlay**: Add cycle-specific messaging and long break support

### State Management Integration

The work cycle system will integrate with the existing state machine:

- **Extended AppState**: Add onboarding and cycle-specific states
- **Enhanced StateManager**: Incorporate cycle orchestration
- **Shared Timer Service**: Extend existing timer for cycle management

### Settings Integration

Onboarding configuration will be stored in the existing settings system:

- **Extended UserSettings**: Add cycle and work schedule fields
- **Settings Migration**: Handle existing users vs new onboarding users
- **Settings UI**: Allow post-onboarding configuration changes

## Platform-Specific Considerations

### Notification Handling

- **macOS**: Use NSUserNotification for system notifications
- **Windows**: Use Windows Toast Notifications
- **Linux**: Use libnotify for desktop notifications

### Work Hours Detection

- **Timezone Handling**: Use system timezone by default
- **Calendar Integration**: Future enhancement to sync with calendar apps
- **Smart Scheduling**: Avoid starting cycles near work day end

## Security and Privacy

### Data Storage

- **Local Storage**: All configuration stored locally in SQLite
- **No Cloud Sync**: Initial version keeps all data on device
- **Encryption**: Sensitive data like emergency keys stored hashed

### Emergency Access

- **Key Combination Security**: Validate emergency keys are not common shortcuts
- **Bypass Logging**: Log emergency exits for user awareness
- **Fail-Safe**: Always allow system-level emergency exits (Ctrl+Alt+Del, etc.)
