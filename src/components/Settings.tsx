import { useSettings, useAppStore } from "../store";

export function Settings() {
  const settings = useSettings();
  const { updateSettings, toggleSettings } = useAppStore();

  return (
    <div className="min-h-screen bg-gray-950 text-white p-8">
      <div className="max-w-4xl mx-auto">
        <div className="flex items-center justify-between mb-8">
          <h1 className="text-3xl font-bold">Pausa Settings</h1>
          <button
            onClick={toggleSettings}
            className="btn-ghost"
            aria-label="Close settings"
          >
            ✕
          </button>
        </div>

        <div className="space-y-6">
          {/* Focus Duration */}
          <div className="settings-section">
            <h2 className="text-xl font-semibold mb-4">Focus Session</h2>
            <div className="space-y-4">
              <div className="settings-item">
                <label className="text-sm font-medium">
                  Focus Duration (minutes)
                </label>
                <input
                  type="number"
                  min="1"
                  max="60"
                  value={settings.focusDuration}
                  onChange={(e) =>
                    updateSettings({ focusDuration: parseInt(e.target.value) })
                  }
                  className="input-primary w-20"
                />
              </div>
            </div>
          </div>

          {/* Break Settings */}
          <div className="settings-section">
            <h2 className="text-xl font-semibold mb-4">Break Settings</h2>
            <div className="space-y-4">
              <div className="settings-item">
                <label className="text-sm font-medium">
                  Short Break (minutes)
                </label>
                <input
                  type="number"
                  min="1"
                  max="30"
                  value={settings.shortBreakDuration}
                  onChange={(e) =>
                    updateSettings({
                      shortBreakDuration: parseInt(e.target.value),
                    })
                  }
                  className="input-primary w-20"
                />
              </div>
              <div className="settings-item">
                <label className="text-sm font-medium">
                  Long Break (minutes)
                </label>
                <input
                  type="number"
                  min="1"
                  max="60"
                  value={settings.longBreakDuration}
                  onChange={(e) =>
                    updateSettings({
                      longBreakDuration: parseInt(e.target.value),
                    })
                  }
                  className="input-primary w-20"
                />
              </div>
              <div className="settings-item">
                <label className="text-sm font-medium">
                  Cycles per Long Break
                </label>
                <input
                  type="number"
                  min="2"
                  max="10"
                  value={settings.cyclesPerLongBreak}
                  onChange={(e) =>
                    updateSettings({
                      cyclesPerLongBreak: parseInt(e.target.value),
                    })
                  }
                  className="input-primary w-20"
                />
              </div>
            </div>
          </div>

          {/* Advanced Settings */}
          <div className="settings-section">
            <h2 className="text-xl font-semibold mb-4">Advanced</h2>
            <div className="space-y-4">
              <div className="settings-item">
                <label className="text-sm font-medium">
                  Pre-Alert (seconds)
                </label>
                <input
                  type="number"
                  min="0"
                  max="300"
                  value={settings.preAlertSeconds}
                  onChange={(e) =>
                    updateSettings({
                      preAlertSeconds: parseInt(e.target.value),
                    })
                  }
                  className="input-primary w-20"
                />
              </div>
              <div className="settings-item">
                <label className="text-sm font-medium">Strict Mode</label>
                <button
                  onClick={() =>
                    updateSettings({ strictMode: !settings.strictMode })
                  }
                  className={`toggle ${settings.strictMode ? "enabled" : ""}`}
                >
                  <span className="toggle-thumb"></span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

