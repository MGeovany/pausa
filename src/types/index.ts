// Core application types based on the design document

export interface UserSettings {
  focusDuration: number; // minutes
  shortBreakDuration: number; // minutes
  longBreakDuration: number; // minutes
  cyclesPerLongBreak: number;
  preAlertSeconds: number;
  strictMode: boolean;
  pinHash?: string;
  emergencyKeyCombination?: string;
  breakTransitionSeconds: number; // seconds before break starts
  blockedApps: string[];
  blockedWebsites: string[];
}

export interface FocusSession {
  id: string;
  startTime: Date;
  duration: number; // seconds
  remaining: number; // seconds
  isRunning: boolean;
  isStrict: boolean;
  state: SessionState;
}

export type SessionState = "idle" | "running" | "pre-alert" | "ending";

export interface BreakSession {
  id: string;
  type: "short" | "long";
  duration: number; // seconds
  remaining: number; // seconds
  activity: BreakActivity;
  allowEmergency: boolean;
}

export interface BreakActivity {
  title: string;
  description: string;
  checklist: string[];
}

export interface SessionStats {
  date: string;
  focusMinutes: number;
  breaksCompleted: number;
  sessionsCompleted: number;
  evasionAttempts: number;
}

export interface Command {
  id: string;
  label: string;
  category: "focus" | "break" | "lock" | "stats" | "settings";
  shortcut?: string;
  action: () => Promise<void>;
}

// Window management types
export type WindowType =
  | "command-palette"
  | "focus-widget"
  | "break-overlay"
  | "settings"
  | "onboarding"
  | "menu-bar-popover"
  | "break-transition"
  | "fullscreen-break-overlay";

// State management types for Zustand
export interface AppState {
  // Current session state
  currentSession: FocusSession | null;
  currentBreak: BreakSession | null;

  // UI state
  isCommandPaletteOpen: boolean;
  isFocusWidgetVisible: boolean;
  isBreakOverlayVisible: boolean;

  // Settings
  settings: UserSettings;

  // Actions
  setCurrentSession: (session: FocusSession | null) => void;
  setCurrentBreak: (breakSession: BreakSession | null) => void;
  toggleCommandPalette: () => void;
  showFocusWidget: () => void;
  hideFocusWidget: () => void;
  showBreakOverlay: () => void;
  hideBreakOverlay: () => void;
  updateSettings: (settings: Partial<UserSettings>) => void;
}

// Toast notification types
export interface Toast {
  id: string;
  type: "success" | "error" | "warning" | "info";
  title: string;
  message?: string;
  duration?: number;
  action?: {
    label: string;
    onClick: () => void;
  };
}

// Error types
export type PausaError =
  | "database-error"
  | "window-management-error"
  | "blocking-service-error"
  | "invalid-state-transition"
  | "authentication-failed"
  | "system-integration-error";

// Tauri command result types
export interface TauriResult<T> {
  success: boolean;
  data?: T;
  error?: {
    type: PausaError;
    message: string;
  };
}

// Event types for real-time updates
export interface SessionUpdateEvent {
  type: "session-update";
  session: FocusSession;
}

export interface BreakUpdateEvent {
  type: "break-update";
  breakSession: BreakSession;
}

export interface StateChangeEvent {
  type: "state-change";
  from: SessionState;
  to: SessionState;
}

export type AppEvent = SessionUpdateEvent | BreakUpdateEvent | StateChangeEvent | CycleEvent;

// Cycle orchestrator types
export type CyclePhase = "idle" | "focus" | "short_break" | "long_break";

export interface CycleState {
  phase: CyclePhase;
  remaining_seconds: number;
  cycle_count: number;
  is_running: boolean;
  can_start: boolean;
  session_id?: string;
  started_at?: string;
}

export interface CycleEvent {
  type: "cycle-event";
  event: CycleEventData;
}

export type CycleEventData =
  | { type: "phase_started"; phase: CyclePhase; duration: number; cycle_count: number }
  | { type: "phase_ended"; phase: CyclePhase; completed: boolean }
  | { type: "tick"; remaining: number }
  | { type: "pre_alert"; remaining: number }
  | { type: "cycle_completed"; cycle_count: number }
  | { type: "long_break_reached"; cycles_completed: number };

// Utility types
export interface Position {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface WindowConfig {
  type: WindowType;
  position?: Position;
  size?: Size;
  alwaysOnTop?: boolean;
  transparent?: boolean;
  clickThrough?: boolean;
  fullscreen?: boolean;
}

// Strict Mode types
export interface StrictModeConfig {
  enabled: boolean;
  emergencyKeyCombination?: string;
  transitionCountdownSeconds: number;
}

export interface StrictModeState {
  isActive: boolean;
  isLocked: boolean;
  currentWindowType?: StrictModeWindowType;
}

export type StrictModeWindowType =
  | "menu_bar_icon"
  | "menu_bar_popover"
  | "break_transition"
  | "fullscreen_break_overlay";
