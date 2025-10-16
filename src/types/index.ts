// Core application types based on the design document

export interface UserSettings {
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
  | "onboarding";

// State management types for Zustand
export interface AppState {
  // Current session state
  currentSession: FocusSession | null;
  currentBreak: BreakSession | null;

  // UI state
  isCommandPaletteOpen: boolean;
  isFocusWidgetVisible: boolean;
  isBreakOverlayVisible: boolean;
  isSettingsOpen: boolean;

  // Settings
  settings: UserSettings;

  // Statistics
  stats: SessionStats[];

  // Actions
  setCurrentSession: (session: FocusSession | null) => void;
  setCurrentBreak: (breakSession: BreakSession | null) => void;
  toggleCommandPalette: () => void;
  showFocusWidget: () => void;
  hideFocusWidget: () => void;
  showBreakOverlay: () => void;
  hideBreakOverlay: () => void;
  toggleSettings: () => void;
  updateSettings: (settings: Partial<UserSettings>) => void;
  setStats: (stats: SessionStats[]) => void;
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

export type AppEvent = SessionUpdateEvent | BreakUpdateEvent | StateChangeEvent;

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
