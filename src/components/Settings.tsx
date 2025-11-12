import { useState } from "react";
import { Plus, Trash2 } from "lucide-react";
import { useSettings, useAppStore } from "../store";

interface SettingsProps {
  onClose?: () => void;
}

export function Settings({ onClose }: SettingsProps) {
  const settings = useSettings();
  const { updateSettings } = useAppStore();
  const [customFocus, setCustomFocus] = useState(settings.focusDuration);
  const [customShortBreak, setCustomShortBreak] = useState(
    settings.shortBreakDuration
  );
  const [customLongBreak, setCustomLongBreak] = useState(
    settings.longBreakDuration
  );
  const [websiteInput, setWebsiteInput] = useState("");
  const [appInput, setAppInput] = useState("");

  const focusOptions = [25, 30, 45];
  const breakOptions = [5, 10, 15];
  const longBreakOptions = [15, 20];
  const cyclesOptions = [3, 4, 5];

  const handleAddWebsite = () => {
    const value = websiteInput.trim();
    if (!value) return;
    const next = Array.from(new Set([...settings.blockedWebsites, value]));
    updateSettings({ blockedWebsites: next });
    setWebsiteInput("");
  };

  const handleRemoveWebsite = (site: string) => {
    updateSettings({
      blockedWebsites: settings.blockedWebsites.filter((item) => item !== site),
    });
  };

  const handleAddApp = () => {
    const value = appInput.trim();
    if (!value) return;
    const next = Array.from(new Set([...settings.blockedApps, value]));
    updateSettings({ blockedApps: next });
    setAppInput("");
  };

  const handleRemoveApp = (app: string) => {
    updateSettings({
      blockedApps: settings.blockedApps.filter((item) => item !== app),
    });
  };

  return (
    <div className="space-y-8">
      <header className="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <div>
          <h1 className="text-3xl font-semibold text-white">Settings</h1>
          <p className="text-sm text-gray-400">
            Tailor focus cycles, breaks, and strict mode to match how you work.
          </p>
        </div>
        {onClose && (
          <button
            onClick={onClose}
            className="rounded-xl border border-gray-800 bg-gray-900/70 px-3 py-2 text-sm text-gray-300 hover:bg-gray-900 hover:text-white transition-colors"
          >
            Close
          </button>
        )}
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
                updateSettings({ focusDuration: option });
              }}
              className={`rounded-xl border px-4 py-2 text-sm transition-colors ${
                settings.focusDuration === option
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
                updateSettings({ focusDuration: value });
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
            selected={settings.shortBreakDuration}
            onSelect={(value) => {
              setCustomShortBreak(value);
              updateSettings({ shortBreakDuration: value });
            }}
            customValue={customShortBreak}
            onCustomChange={(value) => {
              setCustomShortBreak(value);
              updateSettings({ shortBreakDuration: value });
            }}
            suffix="min"
          />
          <SettingTiles
            title="Long break"
            description="Longer pause to fully reset after several cycles."
            options={longBreakOptions}
            selected={settings.longBreakDuration}
            onSelect={(value) => {
              setCustomLongBreak(value);
              updateSettings({ longBreakDuration: value });
            }}
            customValue={customLongBreak}
            onCustomChange={(value) => {
              setCustomLongBreak(value);
              updateSettings({ longBreakDuration: value });
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
                onClick={() => updateSettings({ cyclesPerLongBreak: option })}
                className={`rounded-xl border px-4 py-2 text-sm transition-colors ${
                  settings.cyclesPerLongBreak === option
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
            onClick={() => updateSettings({ strictMode: !settings.strictMode })}
            className={`toggle ${settings.strictMode ? "enabled" : ""}`}
          >
            <span className="toggle-thumb"></span>
          </button>
        </div>

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
              value={settings.preAlertSeconds}
              onChange={(event) =>
                updateSettings({ preAlertSeconds: Number(event.target.value) })
              }
              className="flex-1"
            />
            <span className="w-20 rounded-lg border border-gray-800 bg-gray-900/70 px-2 py-1 text-center text-sm text-white">
              {Math.round(settings.preAlertSeconds / 60)} min
            </span>
          </div>
        </div>
      </section>
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
