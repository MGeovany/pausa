import type { Command } from '../types';
import { useAppStore } from '../store';
import { tauriCommands } from './tauri';

// Command factory functions for creating commands
export const createCommand = (
  id: string,
  label: string,
  category: Command['category'],
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
    'start-focus',
    'Start Focus Session',
    'focus',
    async () => {
      const { settings } = useAppStore.getState();
      await tauriCommands.startFocusSession(settings.strictMode);
    },
    '⌘⇧F'
  );

export const createLockNowCommand = (): Command =>
  createCommand(
    'lock-now',
    'Lock Now',
    'lock',
    async () => {
      // Start an immediate short break
      await tauriCommands.startFocusSession(true);
      await tauriCommands.endSession();
    },
    '⌘⇧L'
  );

export const createHydrateBreakCommand = (): Command =>
  createCommand(
    'hydrate-break',
    'Take Hydrate Break',
    'break',
    async () => {
      // Start a short break session
      await tauriCommands.startFocusSession(false);
      await tauriCommands.endSession();
    }
  );

export const createStatsCommand = (): Command =>
  createCommand(
    'stats',
    'View Statistics',
    'stats',
    async () => {
      window.location.hash = '#/stats';
    }
  );

export const createSettingsCommand = (): Command =>
  createCommand(
    'settings',
    'Open Settings',
    'settings',
    async () => {
      window.location.hash = '#/settings';
    }
  );

export const createBreakActivitiesCommand = (): Command =>
  createCommand(
    'break-activities',
    'Manage Break Activities',
    'settings',
    async () => {
      // This will be handled by the parent component
      console.log('Opening break activities settings...');
    }
  );

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
  }
};

// Command registry with search indexing
export class CommandRegistry {
  private commands: Command[] = [];
  private searchIndex: Map<string, Command[]> = new Map();

  constructor() {
    this.registerDefaultCommands();
    this.buildSearchIndex();
  }

  private registerDefaultCommands() {
    this.commands = [
      createStartFocusCommand(),
      createLockNowCommand(),
      createHydrateBreakCommand(),
      createStatsCommand(),
      createSettingsCommand(),
      createBreakActivitiesCommand(),
    ];
  }

  private buildSearchIndex() {
    this.searchIndex.clear();

    this.commands.forEach(command => {
      // Index by label words
      const labelWords = command.label.toLowerCase().split(' ');
      labelWords.forEach(word => {
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
    return [...this.commands];
  }

  searchCommands(query: string): Command[] {
    if (!query.trim()) {
      return this.getAllCommands();
    }

    const normalizedQuery = query.toLowerCase().trim();
    const results = new Set<Command>();

    // Exact matches first
    if (this.searchIndex.has(normalizedQuery)) {
      this.searchIndex.get(normalizedQuery)!.forEach(cmd => results.add(cmd));
    }

    // Partial matches
    for (const [key, commands] of this.searchIndex.entries()) {
      if (key.includes(normalizedQuery) || normalizedQuery.includes(key)) {
        commands.forEach(cmd => results.add(cmd));
      }
    }

    // Fuzzy matching for remaining commands
    this.commands.forEach(command => {
      if (!results.has(command)) {
        const label = command.label.toLowerCase();
        const category = command.category.toLowerCase();

        // Simple fuzzy matching - check if query characters appear in order
        if (this.fuzzyMatch(normalizedQuery, label) ||
          this.fuzzyMatch(normalizedQuery, category)) {
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
    this.commands.push(command);
    this.buildSearchIndex();
  }

  removeCommand(commandId: string) {
    this.commands = this.commands.filter(cmd => cmd.id !== commandId);
    this.buildSearchIndex();
  }

  getCommand(commandId: string): Command | undefined {
    return this.commands.find(cmd => cmd.id === commandId);
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
