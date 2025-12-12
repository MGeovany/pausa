import type { Command } from "../types";
import { useAppStore } from "../store";
import { tauriCommands } from "./tauri";

// Command factory functions for creating commands
export const createCommand = (
  id: string,
  label: string,
  category: Command["category"],
  action: () => Promise<void>,
  shortcut?: string
): Command => ({
  id,
  label,
  category,
  action,
  shortcut,
});

// Command implementations
export const createStartFocusCommand = (): Command =>
  createCommand(
    "start-focus",
    "Start Focus Session",
    "focus",
    async () => {
      const { settings } = useAppStore.getState();
      await tauriCommands.startFocusSession(settings.strictMode);
    },
    "⌘⇧F"
  );

export const createHydrateBreakCommand = (): Command =>
  createCommand("hydrate-break", "Take Hydrate Break", "break", async () => {
    // Start a short break session
    await tauriCommands.startFocusSession(false);
    await tauriCommands.endSession();
  });

export const createStatsCommand = (
  navigateFn?: (path: string) => void
): Command =>
  createCommand("stats", "View Statistics", "stats", async () => {
    // Navigate to stats page
    if (navigateFn) {
      navigateFn("/stats");
    } else {
      // Fallback to hash navigation
      if (window.location.hash !== "#/stats") {
        window.location.hash = "#/stats";
      } else {
        // Force a re-render if we're already there
        window.dispatchEvent(new PopStateEvent("popstate"));
      }
    }
  });

export const createSettingsCommand = (): Command =>
  createCommand("settings", "Open Settings", "settings", async () => {
    window.location.hash = "#/settings";
  });

export const createBreakActivitiesCommand = (): Command =>
  createCommand(
    "break-activities",
    "Manage Break Activities",
    "settings",
    async () => {
      // This will be handled by the parent component
      console.log("Opening break activities settings...");
    }
  );

// Pause command - only available when session is running
export const createPauseCommand = (onPause: () => Promise<void>): Command =>
  createCommand("pause-session", "Pause Session", "focus", async () => {
    await onPause();
  });

// Resume command - only available when session is paused
export const createResumeCommand = (onResume: () => Promise<void>): Command =>
  createCommand("resume-session", "Resume Session", "focus", async () => {
    await onResume();
  });

// End session command - only available when session is active
export const createEndSessionCommand = (onEnd: () => Promise<void>): Command =>
  createCommand("end-session", "End Session", "focus", async () => {
    await onEnd();
  });

// Session management functions for use in components
export const SessionManager = {
  async getCurrentSession() {
    return await tauriCommands.getCurrentSession();
  },

  async getCurrentBreak() {
    return await tauriCommands.getCurrentBreak();
  },

  async toggleSession() {
    const currentSession = await tauriCommands.getCurrentSession();
    if (currentSession) {
      if (currentSession.isRunning) {
        await tauriCommands.pauseSession();
      } else {
        await tauriCommands.resumeSession();
      }
    } else {
      // Start new session with current settings
      const { settings } = useAppStore.getState();
      await tauriCommands.startFocusSession(settings.strictMode);
    }
  },

  async resetSession() {
    await tauriCommands.endSession();
  },

  async startFocusSession(strict: boolean = false) {
    return await tauriCommands.startFocusSession(strict);
  },

  async endSession() {
    await tauriCommands.endSession();
  },

  async completeBreak() {
    await tauriCommands.completeBreak();
  },
};

// Command registry with search indexing
export class CommandRegistry {
  private baseCommands: Command[] = [];
  private dynamicCommands: Command[] = [];
  private searchIndex: Map<string, Command[]> = new Map();
  private navigateFn?: (path: string) => void;

  constructor() {
    this.registerDefaultCommands();
    this.buildSearchIndex();
  }

  private registerDefaultCommands() {
    this.baseCommands = [
      createStartFocusCommand(),
      createStatsCommand(this.navigateFn),
      createSettingsCommand(),
    ];
  }

  // Re-register commands with navigation function
  registerWithNavigation(navigateFn: (path: string) => void) {
    this.navigateFn = navigateFn;
    this.registerDefaultCommands();
    this.buildSearchIndex();
  }

  // Update dynamic commands based on cycle state
  updateDynamicCommands(
    cycleState: any,
    handlers: {
      onPause: () => Promise<void>;
      onResume: () => Promise<void>;
      onEnd: () => Promise<void>;
    }
  ) {
    this.dynamicCommands = [];

    // Only add pause/resume/end commands if there's an active session
    if (cycleState && cycleState.phase !== "idle") {
      if (cycleState.is_running) {
        // Session is running, show pause
        this.dynamicCommands.push(createPauseCommand(handlers.onPause));
      } else {
        // Session is paused, show resume
        this.dynamicCommands.push(createResumeCommand(handlers.onResume));
      }
      // Always show end when there's an active session
      this.dynamicCommands.push(createEndSessionCommand(handlers.onEnd));
    }

    this.buildSearchIndex();
  }

  private getAllCommandsInternal(): Command[] {
    return [...this.baseCommands, ...this.dynamicCommands];
  }

  private buildSearchIndex() {
    this.searchIndex.clear();
    const allCommands = this.getAllCommandsInternal();

    allCommands.forEach((command) => {
      // Index by label words
      const labelWords = command.label.toLowerCase().split(" ");
      labelWords.forEach((word) => {
        if (!this.searchIndex.has(word)) {
          this.searchIndex.set(word, []);
        }
        this.searchIndex.get(word)!.push(command);
      });

      // Index by category
      const category = command.category.toLowerCase();
      if (!this.searchIndex.has(category)) {
        this.searchIndex.set(category, []);
      }
      this.searchIndex.get(category)!.push(command);

      // Index by full label
      const fullLabel = command.label.toLowerCase();
      if (!this.searchIndex.has(fullLabel)) {
        this.searchIndex.set(fullLabel, []);
      }
      this.searchIndex.get(fullLabel)!.push(command);
    });
  }

  getAllCommands(): Command[] {
    return this.getAllCommandsInternal();
  }

  searchCommands(query: string): Command[] {
    const allCommands = this.getAllCommandsInternal();
    if (!query.trim()) {
      return allCommands;
    }

    const normalizedQuery = query.toLowerCase().trim();
    const results = new Set<Command>();

    // Exact matches first
    if (this.searchIndex.has(normalizedQuery)) {
      this.searchIndex.get(normalizedQuery)!.forEach((cmd) => results.add(cmd));
    }

    // Partial matches
    for (const [key, commands] of this.searchIndex.entries()) {
      if (key.includes(normalizedQuery) || normalizedQuery.includes(key)) {
        commands.forEach((cmd) => results.add(cmd));
      }
    }

    // Fuzzy matching for remaining commands
    allCommands.forEach((command) => {
      if (!results.has(command)) {
        const label = command.label.toLowerCase();
        const category = command.category.toLowerCase();

        // Simple fuzzy matching - check if query characters appear in order
        if (
          this.fuzzyMatch(normalizedQuery, label) ||
          this.fuzzyMatch(normalizedQuery, category)
        ) {
          results.add(command);
        }
      }
    });

    return Array.from(results);
  }

  private fuzzyMatch(query: string, target: string): boolean {
    let queryIndex = 0;
    let targetIndex = 0;

    while (queryIndex < query.length && targetIndex < target.length) {
      if (query[queryIndex] === target[targetIndex]) {
        queryIndex++;
      }
      targetIndex++;
    }

    return queryIndex === query.length;
  }

  addCommand(command: Command) {
    this.baseCommands.push(command);
    this.buildSearchIndex();
  }

  removeCommand(commandId: string) {
    this.baseCommands = this.baseCommands.filter((cmd) => cmd.id !== commandId);
    this.buildSearchIndex();
  }

  getCommand(commandId: string): Command | undefined {
    const allCommands = this.getAllCommandsInternal();
    return allCommands.find((cmd) => cmd.id === commandId);
  }
}

// Global command registry instance
export const commandRegistry = new CommandRegistry();

// Hook for using commands in React components
export const useCommands = () => {
  return {
    getAllCommands: () => commandRegistry.getAllCommands(),
    searchCommands: (query: string) => commandRegistry.searchCommands(query),
    executeCommand: async (commandId: string) => {
      const command = commandRegistry.getCommand(commandId);
      if (command) {
        try {
          await command.action();
        } catch (error) {
          console.error(`Failed to execute command ${commandId}:`, error);
          throw error;
        }
      } else {
        throw new Error(`Command not found: ${commandId}`);
      }
    },
  };
};
