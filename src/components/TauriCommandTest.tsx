import React, { useState } from 'react';
import { tauriCommands } from '../lib/tauri';
import type { FocusSession, UserSettings, SessionStats } from '../types';

export const TauriCommandTest: React.FC = () => {
  const [output, setOutput] = useState<string>('');
  const [currentSession, setCurrentSession] = useState<FocusSession | null>(null);
  const [settings, setSettings] = useState<UserSettings | null>(null);
  const [stats, setStats] = useState<SessionStats[]>([]);

  const log = (message: string) => {
    setOutput(prev => prev + '\n' + message);
    console.log(message);
  };

  const testCommands = async () => {
    try {
      log('=== Testing Tauri Commands ===');

      // Test getting initial state
      log('Testing get_app_state...');
      const appState = await tauriCommands.getAppState();
      log(`App State: ${appState}`);

      // Test getting settings
      log('Testing get_settings...');
      const currentSettings = await tauriCommands.getSettings();
      setSettings(currentSettings);
      log(`Settings: ${JSON.stringify(currentSettings, null, 2)}`);

      // Test getting current session (should be null initially)
      log('Testing get_current_session...');
      const initialSession = await tauriCommands.getCurrentSession();
      log(`Initial Session: ${initialSession ? 'Found session' : 'No session'}`);

      // Test starting a focus session
      log('Testing start_focus_session...');
      const newSession = await tauriCommands.startFocusSession(false);
      setCurrentSession(newSession);
      log(`Started Session: ${newSession.id}, Duration: ${newSession.duration}s`);

      // Test getting current session again
      log('Testing get_current_session after start...');
      const activeSession = await tauriCommands.getCurrentSession();
      log(`Active Session: ${activeSession ? activeSession.id : 'No session'}`);

      // Test pausing session
      log('Testing pause_session...');
      await tauriCommands.pauseSession();
      log('Session paused');

      // Test resuming session
      log('Testing resume_session...');
      await tauriCommands.resumeSession();
      log('Session resumed');

      // Test getting session stats
      log('Testing get_session_stats...');
      const sessionStats = await tauriCommands.getSessionStats(7);
      setStats(sessionStats);
      log(`Session Stats: ${sessionStats.length} entries`);

      // Test ending session
      log('Testing end_session...');
      await tauriCommands.endSession();
      log('Session ended');

      // Test database stats
      log('Testing get_database_stats...');
      const dbStats = await tauriCommands.getDatabaseStats();
      log(`Database Stats: ${dbStats}`);

      // Test state manager
      log('Testing test_state_manager...');
      const stateManagerTest = await tauriCommands.testStateManager();
      log(`State Manager Test: ${stateManagerTest}`);

      log('=== All tests completed successfully! ===');

    } catch (error) {
      log(`Error: ${error}`);
    }
  };

  const updateTestSettings = async () => {
    if (!settings) return;

    try {
      log('Testing update_settings...');
      const newSettings = {
        ...settings,
        focusDuration: 30, // Change from 25 to 30 minutes
        strictMode: true,
      };

      await tauriCommands.updateSettings(newSettings);
      log('Settings updated successfully');

      // Verify the update
      const updatedSettings = await tauriCommands.getSettings();
      setSettings(updatedSettings);
      log(`Updated Settings: Focus Duration = ${updatedSettings.focusDuration}, Strict Mode = ${updatedSettings.strictMode}`);

    } catch (error) {
      log(`Settings update error: ${error}`);
    }
  };

  return (
    <div className="p-6 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-4">Tauri Command Test</h1>
      
      <div className="space-y-4">
        <button
          onClick={testCommands}
          className="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
        >
          Run All Tests
        </button>

        <button
          onClick={updateTestSettings}
          disabled={!settings}
          className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 disabled:opacity-50"
        >
          Test Settings Update
        </button>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <h3 className="text-lg font-semibold mb-2">Current Session</h3>
            <pre className="bg-gray-100 p-3 rounded text-sm overflow-auto">
              {currentSession ? JSON.stringify(currentSession, null, 2) : 'No active session'}
            </pre>
          </div>

          <div>
            <h3 className="text-lg font-semibold mb-2">Settings</h3>
            <pre className="bg-gray-100 p-3 rounded text-sm overflow-auto">
              {settings ? JSON.stringify(settings, null, 2) : 'No settings loaded'}
            </pre>
          </div>
        </div>

        <div>
          <h3 className="text-lg font-semibold mb-2">Session Stats</h3>
          <pre className="bg-gray-100 p-3 rounded text-sm overflow-auto">
            {stats.length > 0 ? JSON.stringify(stats, null, 2) : 'No stats available'}
          </pre>
        </div>

        <div>
          <h3 className="text-lg font-semibold mb-2">Test Output</h3>
          <pre className="bg-black text-green-400 p-4 rounded text-sm h-96 overflow-auto">
            {output || 'Click "Run All Tests" to start testing...'}
          </pre>
        </div>
      </div>
    </div>
  );
};
