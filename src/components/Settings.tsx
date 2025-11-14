import { useState, useEffect, useRef } from "react";
import { Plus, Trash2, Save, X } from "lucide-react";
import { useSettings, useAppStore } from "../store";
import { tauriCommands } from "../lib/tauri";
import type { UserSettings } from "../types";

interface SettingsProps {
  onClose?: () => void;
}

export function Settings({ onClose }: SettingsProps) {
  const initialSettings = useSettings();
  const { updateSettings } = useAppStore();

  // Local state for all settings
  const [localSettings, setLocalSettings] =
    useState<UserSettings>(initialSettings);
  const [customFocus, setCustomFocus] = useState(initialSettings.focusDuration);
  const [customShortBreak, setCustomShortBreak] = useState(
    initialSettings.shortBreakDuration
  );
  const [customLongBreak, setCustomLongBreak] = useState(
    initialSettings.longBreakDuration
  );
  const [websiteInput, setWebsiteInput] = useState("");
  const [appInput, setAppInput] = useState("");
  const [isSaving, setIsSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);
  const [showEmergencyKeyModal, setShowEmergencyKeyModal] = useState(false);
  const [emergencyKey, setEmergencyKey] = useState("");
  const [isCapturingKey, setIsCapturingKey] = useState(false);
  const pressedKeysRef = useRef<Set<string>>(new Set());

  // Update local settings when initial settings change
  useEffect(() => {
    setLocalSettings(initialSettings);
    setCustomFocus(initialSettings.focusDuration);
    setCustomShortBreak(initialSettings.shortBreakDuration);
    setCustomLongBreak(initialSettings.longBreakDuration);
    setEmergencyKey(initialSettings.emergencyKeyCombination || "");
  }, [initialSettings]);

  // Key capture logic for emergency key combination
  useEffect(() => {
    if (!isCapturingKey || !showEmergencyKeyModal) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      // Ignore repeated keydown events (when key is held down)
      if (event.repeat) return;

      const pressedKeys = pressedKeysRef.current;

      // Add modifier keys to the set
      if (event.metaKey && !pressedKeys.has("⌘")) {
        pressedKeys.add("⌘");
      }
      if (event.ctrlKey && !pressedKeys.has("Ctrl")) {
        pressedKeys.add("Ctrl");
      }
      if (event.altKey && !pressedKeys.has("Alt")) {
        pressedKeys.add("Alt");
      }
      if (event.shiftKey && !pressedKeys.has("⇧")) {
        pressedKeys.add("⇧");
      }

      // Add the main key (if it's not a modifier)
      if (!["Meta", "Control", "Alt", "Shift"].includes(event.key)) {
        const mainKey = event.key.toUpperCase();
        if (!pressedKeys.has(mainKey)) {
          pressedKeys.add(mainKey);
        }
      }

      // Update the display with all pressed keys
      const keysArray = Array.from(pressedKeys);
      if (keysArray.length > 0) {
        const keyCombo = keysArray.join(" + ");
        setEmergencyKey(keyCombo);
      }
    };

    const handleKeyUp = (event: KeyboardEvent) => {
      event.preventDefault();
      event.stopPropagation();

      // Stop capturing when all keys are released
      if (
        !event.metaKey &&
        !event.ctrlKey &&
        !event.altKey &&
        !event.shiftKey
      ) {
        if (pressedKeysRef.current.size > 0) {
          setIsCapturingKey(false);
        }
      }
    };

    document.addEventListener("keydown", handleKeyDown, true);
    document.addEventListener("keyup", handleKeyUp, true);

    return () => {
      document.removeEventListener("keydown", handleKeyDown, true);
      document.removeEventListener("keyup", handleKeyUp, true);
    };
  }, [isCapturingKey, showEmergencyKeyModal]);

  const focusOptions = [25, 30, 45];
  const breakOptions = [5, 10, 15];
  const longBreakOptions = [15, 20];
  const cyclesOptions = [3, 4, 5];

  const updateLocalSettings = (updates: Partial<UserSettings>) => {
    setLocalSettings((prev) => ({ ...prev, ...updates }));
  };

  const handleAddWebsite = () => {
    const value = websiteInput.trim();
    if (!value) return;
    const next = Array.from(new Set([...localSettings.blockedWebsites, value]));
    updateLocalSettings({ blockedWebsites: next });
    setWebsiteInput("");
  };

  const handleRemoveWebsite = (site: string) => {
    updateLocalSettings({
      blockedWebsites: localSettings.blockedWebsites.filter(
        (item) => item !== site
      ),
    });
  };

  const handleAddApp = () => {
    const value = appInput.trim();
    if (!value) return;
    const next = Array.from(new Set([...localSettings.blockedApps, value]));
    updateLocalSettings({ blockedApps: next });
    setAppInput("");
  };

  const handleRemoveApp = (app: string) => {
    updateLocalSettings({
      blockedApps: localSettings.blockedApps.filter((item) => item !== app),
    });
  };

  const handleStrictModeToggle = (enabled: boolean) => {
    if (enabled && !localSettings.emergencyKeyCombination) {
      // If enabling strict mode and no emergency key is set, show modal
      setShowEmergencyKeyModal(true);
      setEmergencyKey("");
    } else {
      updateLocalSettings({ strictMode: enabled });
    }
  };

  const startKeyCapture = () => {
    setIsCapturingKey(true);
    pressedKeysRef.current.clear();
    setEmergencyKey("");
  };

  const clearKeyCapture = () => {
    setIsCapturingKey(false);
    pressedKeysRef.current.clear();
    setEmergencyKey("");
  };

  const handleSaveEmergencyKey = () => {
    if (emergencyKey.trim()) {
      updateLocalSettings({
        strictMode: true,
        emergencyKeyCombination: emergencyKey.trim(),
      });
      setShowEmergencyKeyModal(false);
      setIsCapturingKey(false);
      pressedKeysRef.current.clear();
    }
  };

  const handleCancelEmergencyKey = () => {
    setShowEmergencyKeyModal(false);
    setIsCapturingKey(false);
    pressedKeysRef.current.clear();
    setEmergencyKey("");
    // Revert strict mode toggle
    updateLocalSettings({ strictMode: false });
  };

  const handleSave = async () => {
    setIsSaving(true);
    setSaveMessage(null);

    try {
      // Save to backend
      await tauriCommands.updateSettings(localSettings);

      // Update store
      updateSettings(localSettings);

      setSaveMessage({ type: "success", text: "Settings saved successfully!" });

      // Clear message after 3 seconds
      setTimeout(() => {
        setSaveMessage(null);
      }, 3000);
    } catch (error) {
      console.error("Failed to save settings:", error);
      setSaveMessage({
        type: "error",
        text: "Failed to save settings. Please try again.",
      });
    } finally {
      setIsSaving(false);
    }
  };

  const hasChanges =
    JSON.stringify(localSettings) !== JSON.stringify(initialSettings);

  return (
    <div className="space-y-8">
      <header className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-white">Settings</h1>
          <p className="text-sm text-gray-400">
            Tailor focus cycles, breaks, and strict mode to match how you work.
          </p>
        </div>
        <div className="flex items-center gap-3">
          {hasChanges && (
            <button
              onClick={handleSave}
              disabled={isSaving}
              className="flex items-center gap-2 rounded-xl border border-blue-500 bg-blue-500/20 px-4 py-2 text-sm text-blue-200 hover:bg-blue-500/30 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              <Save className="h-4 w-4" />
              {isSaving ? "Saving..." : "Save"}
            </button>
          )}
          {saveMessage && (
            <div
              className={`rounded-xl px-4 py-2 text-sm ${
                saveMessage.type === "success"
                  ? "bg-green-500/20 text-green-200 border border-green-500/40"
                  : "bg-red-500/20 text-red-200 border border-red-500/40"
              }`}
            >
              {saveMessage.text}
            </div>
          )}
          {onClose && (
            <button
              onClick={onClose}
              className="rounded-xl border border-gray-800 bg-gray-900/70 px-3 py-2 text-sm text-gray-300 hover:bg-gray-900 hover:text-white transition-colors"
            >
              Close
            </button>
          )}
        </div>
      </header>

      <section className="rounded-2xl border border-gray-800 bg-gray-900/60 p-6">
        <h2 className="text-xl font-semibold text-white">Focus sessions</h2>
        <p className="text-sm text-gray-500">
          Pick the ideal length for deep-work blocks or store a custom value
          that fits your rhythm.
        </p>
        <div className="mt-6 flex flex-wrap gap-3">
          {focusOptions.map((option) => (
            <button
              key={option}
              onClick={() => {
                setCustomFocus(option);
                updateLocalSettings({ focusDuration: option });
              }}
              className={`rounded-xl border px-4 py-2 text-sm transition-colors ${
                localSettings.focusDuration === option
                  ? "border-blue-500 bg-blue-500/20 text-blue-200"
                  : "border-gray-700 bg-gray-800/60 text-gray-300 hover:border-blue-500/40 hover:text-white"
              }`}
            >
              {option} min
            </button>
          ))}
          <div className="flex items-center gap-2 rounded-xl border border-gray-800 bg-gray-950/60 px-3 py-2">
            <span className="text-xs uppercase tracking-wide text-gray-500">
              Custom
            </span>
            <input
              type="number"
              min={10}
              max={120}
              value={customFocus}
              onChange={(event) => {
                const value = Number(event.target.value);
                setCustomFocus(value);
                updateLocalSettings({ focusDuration: value });
              }}
              className="w-16 rounded-lg border border-gray-700 bg-gray-900 px-2 py-1 text-sm text-white focus:border-blue-500 focus:outline-none"
            />
            <span className="text-xs text-gray-500">min</span>
          </div>
        </div>
      </section>

      <section className="rounded-2xl border border-gray-800 bg-gray-900/60 p-6">
        <h2 className="text-xl font-semibold text-white">Breaks</h2>
        <div className="mt-6 grid gap-6 md:grid-cols-2">
          <SettingTiles
            title="Short break"
            description="Quick pause to reset energy between focus blocks."
            options={breakOptions}
            selected={localSettings.shortBreakDuration}
            onSelect={(value) => {
              setCustomShortBreak(value);
              updateLocalSettings({ shortBreakDuration: value });
            }}
            customValue={customShortBreak}
            onCustomChange={(value) => {
              setCustomShortBreak(value);
              updateLocalSettings({ shortBreakDuration: value });
            }}
            suffix="min"
          />
          <SettingTiles
            title="Long break"
            description="Longer pause to fully reset after several cycles."
            options={longBreakOptions}
            selected={localSettings.longBreakDuration}
            onSelect={(value) => {
              setCustomLongBreak(value);
              updateLocalSettings({ longBreakDuration: value });
            }}
            customValue={customLongBreak}
            onCustomChange={(value) => {
              setCustomLongBreak(value);
              updateLocalSettings({ longBreakDuration: value });
            }}
            suffix="min"
          />
        </div>
        <div className="mt-6">
          <h3 className="text-sm font-medium text-gray-300">
            Cycles per long break
          </h3>
          <div className="mt-3 flex flex-wrap gap-2">
            {cyclesOptions.map((option) => (
              <button
                key={option}
                onClick={() =>
                  updateLocalSettings({ cyclesPerLongBreak: option })
                }
                className={`rounded-xl border px-4 py-2 text-sm transition-colors ${
                  localSettings.cyclesPerLongBreak === option
                    ? "border-purple-500 bg-purple-500/20 text-purple-200"
                    : "border-gray-700 bg-gray-800/60 text-gray-300 hover:border-purple-500/40 hover:text-white"
                }`}
              >
                {option} cycles
              </button>
            ))}
          </div>
        </div>
      </section>

      <section className="rounded-2xl border border-gray-800 bg-gray-900/60 p-6">
        <h2 className="text-xl font-semibold text-white">Strict mode</h2>
        <p className="text-sm text-gray-500">
          Keep distractions out of reach while you are in focus mode.
        </p>

        <div className="mt-6 flex items-center justify-between rounded-xl border border-gray-800 bg-gray-950/60 p-4">
          <div>
            <h3 className="text-sm font-semibold text-white">
              Enable strict mode
            </h3>
            <p className="text-xs text-gray-500">
              Locks the screen and blocks anything on your list while a focus
              session is running.
            </p>
          </div>
          <button
            onClick={() => handleStrictModeToggle(!localSettings.strictMode)}
            className={`toggle ${localSettings.strictMode ? "enabled" : ""}`}
          >
            <span className="toggle-thumb"></span>
          </button>
        </div>
        {localSettings.strictMode && localSettings.emergencyKeyCombination && (
          <div className="mt-6 rounded-xl border border-gray-800 bg-gray-950/60 p-4">
            <h3 className="text-sm font-semibold text-white mb-2">
              Emergency Key Combination
            </h3>
            <div className="flex items-center justify-between">
              <p className="text-sm text-gray-300">
                {localSettings.emergencyKeyCombination}
              </p>
              <button
                onClick={() => {
                  setEmergencyKey(localSettings.emergencyKeyCombination || "");
                  setShowEmergencyKeyModal(true);
                }}
                className="text-xs text-blue-400 hover:text-blue-300 transition-colors"
              >
                Change
              </button>
            </div>
          </div>
        )}
        <div className="mt-6 rounded-xl border border-gray-800 bg-gray-950/60 p-4">
          <h3 className="text-sm font-semibold text-white">Pre-alert</h3>
          <p className="text-xs text-gray-500">
            Receive an early warning before your focus block finishes.
          </p>
          <div className="mt-3 flex items-center gap-3">
            <input
              type="range"
              min={0}
              max={300}
              step={15}
              value={localSettings.preAlertSeconds}
              onChange={(event) =>
                updateLocalSettings({
                  preAlertSeconds: Number(event.target.value),
                })
              }
              className="flex-1"
            />
            <span className="w-20 rounded-lg border border-gray-800 bg-gray-900/70 px-2 py-1 text-center text-sm text-white">
              {Math.round(localSettings.preAlertSeconds / 60)} min
            </span>
          </div>
        </div>
      </section>

      {/* Emergency Key Combination Modal */}
      {showEmergencyKeyModal && (
        <div className="fixed inset-0 z-50 bg-black/70 flex items-center justify-center p-4">
          <div className="bg-gray-800 rounded-2xl p-6 max-w-md w-full border border-gray-700">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-xl font-semibold text-white">
                Emergency Key Combination
              </h3>
              <button
                onClick={handleCancelEmergencyKey}
                className="text-gray-400 hover:text-white transition-colors"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            <p className="text-gray-300 mb-6 text-sm">
              Configure a key combination to exit strict mode in case of
              emergency. This combination will be required to unlock the screen
              during focus sessions.
            </p>

            <div className="space-y-4">
              <div>
                <label className="block text-gray-300 text-sm font-medium mb-2">
                  Key Combination
                </label>
                <div className="flex space-x-3">
                  <div className="flex-1">
                    <input
                      type="text"
                      value={emergencyKey}
                      readOnly
                      placeholder="Press your combination to exit strict mode"
                      className="w-full p-3 bg-gray-900/50 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:border-blue-500 focus:outline-none"
                    />
                  </div>
                  <button
                    onClick={startKeyCapture}
                    disabled={isCapturingKey}
                    className={`px-4 py-3 rounded-lg font-medium transition-all ${
                      isCapturingKey
                        ? "bg-yellow-600 text-yellow-100 cursor-not-allowed"
                        : "bg-yellow-700 text-yellow-100 hover:bg-yellow-600"
                    }`}
                  >
                    {isCapturingKey ? "Press keys..." : "Capture"}
                  </button>
                  {emergencyKey && (
                    <button
                      onClick={clearKeyCapture}
                      className="px-4 py-3 rounded-lg font-medium bg-gray-700 text-gray-300 hover:bg-gray-600 transition-all"
                    >
                      Clear
                    </button>
                  )}
                </div>
              </div>

              <div className="bg-gray-900/30 rounded-lg p-4">
                <p className="text-gray-400 text-xs mb-2">
                  <strong className="text-gray-300">Example:</strong> ⌘ + ⇧ + P
                </p>
                <p className="text-gray-500 text-xs">
                  Choose something you'll remember but others won't guess.
                </p>
              </div>

              {isCapturingKey && (
                <div className="bg-blue-900/20 border border-blue-700/50 rounded-lg p-4">
                  <p className="text-blue-200 text-sm">
                    🎯 Press your desired key combination now...
                  </p>
                </div>
              )}

              {!emergencyKey && (
                <div className="bg-red-900/20 border border-red-700/50 rounded-lg p-4">
                  <p className="text-red-200 text-sm">
                    ⚠️ Emergency key combination is required when strict mode is
                    enabled.
                  </p>
                </div>
              )}
            </div>

            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={handleCancelEmergencyKey}
                className="px-4 py-2 rounded-lg font-medium bg-gray-700 text-gray-300 hover:bg-gray-600 transition-all"
              >
                Cancel
              </button>
              <button
                onClick={handleSaveEmergencyKey}
                disabled={!emergencyKey.trim()}
                className={`px-4 py-2 rounded-lg font-medium transition-all ${
                  emergencyKey.trim()
                    ? "bg-blue-600 text-white hover:bg-blue-700"
                    : "bg-gray-600 text-gray-400 cursor-not-allowed"
                }`}
              >
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

interface SettingTilesProps {
  title: string;
  description: string;
  options: number[];
  selected: number;
  onSelect: (value: number) => void;
  customValue: number;
  onCustomChange: (value: number) => void;
  suffix?: string;
}

function SettingTiles({
  title,
  description,
  options,
  selected,
  onSelect,
  customValue,
  onCustomChange,
  suffix = "",
}: SettingTilesProps) {
  return (
    <div>
      <h3 className="text-lg font-semibold text-white">{title}</h3>
      <p className="text-xs text-gray-500">{description}</p>
      <div className="mt-4 flex flex-wrap gap-2">
        {options.map((option) => (
          <button
            key={option}
            onClick={() => onSelect(option)}
            className={`rounded-xl border px-4 py-2 text-sm transition-colors ${
              selected === option
                ? "border-green-500 bg-green-500/20 text-green-200"
                : "border-gray-700 bg-gray-800/60 text-gray-300 hover:border-green-500/40 hover:text-white"
            }`}
          >
            {option} {suffix}
          </button>
        ))}
        <div className="flex items-center gap-2 rounded-xl border border-gray-800 bg-gray-950/60 px-3 py-2">
          <span className="text-xs uppercase tracking-wide text-gray-500">
            Custom
          </span>
          <input
            type="number"
            min={1}
            max={60}
            value={customValue}
            onChange={(event) => onCustomChange(Number(event.target.value))}
            className="w-16 rounded-lg border border-gray-700 bg-gray-900 px-2 py-1 text-sm text-white focus:border-green-500 focus:outline-none"
          />
          <span className="text-xs text-gray-500">{suffix}</span>
        </div>
      </div>
    </div>
  );
}

interface BlockListProps {
  title: string;
  placeholder: string;
  value: string;
  onChange: (value: string) => void;
  items: string[];
  onAdd: () => void;
  onRemove: (value: string) => void;
}

function BlockList({
  title,
  placeholder,
  value,
  onChange,
  items,
  onAdd,
  onRemove,
}: BlockListProps) {
  return (
    <div>
      <h3 className="text-lg font-semibold text-white">{title}</h3>
      <p className="text-xs text-gray-500">
        Add items that should be blocked automatically while strict mode is
        active.
      </p>
      <div className="mt-4 flex items-center gap-2">
        <div className="flex-1 rounded-xl border border-gray-800 bg-gray-950/60 px-3 py-2">
          <input
            value={value}
            onChange={(event) => onChange(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") {
                event.preventDefault();
                onAdd();
              }
            }}
            placeholder={placeholder}
            className="w-full bg-transparent text-sm text-white focus:outline-none"
          />
        </div>
        <button
          onClick={onAdd}
          className="flex items-center gap-2 rounded-xl border border-blue-500/40 bg-blue-500/10 px-3 py-2 text-sm text-blue-200 hover:bg-blue-500/20 transition-colors"
        >
          <Plus className="h-4 w-4" />
          Add
        </button>
      </div>
      <ul className="mt-4 space-y-2">
        {items.map((item) => (
          <li
            key={item}
            className="flex items-center justify-between rounded-xl border border-gray-800 bg-gray-950/60 px-3 py-2 text-sm text-gray-300"
          >
            <span>{item}</span>
            <button
              onClick={() => onRemove(item)}
              className="rounded-lg border border-red-500/40 bg-red-500/10 p-1 text-red-200 hover:bg-red-500/20 transition-colors"
              aria-label={`Eliminar ${item}`}
            >
              <Trash2 className="h-4 w-4" />
            </button>
          </li>
        ))}
        {items.length === 0 && (
          <li className="rounded-xl border border-dashed border-gray-800 bg-gray-900/40 px-3 py-4 text-center text-xs text-gray-500">
            You have no blocked entries yet.
          </li>
        )}
      </ul>
    </div>
  );
}
