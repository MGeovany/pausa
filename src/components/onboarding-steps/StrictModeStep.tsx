import { useState, useEffect, useRef } from "react";
import type { StepProps } from "./types";

interface StrictModeConfig {
  strictMode: boolean;
  emergencyKey: string;
  userName: string;
}

export default function StrictModeStep({
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
  stepData,
  setStepData,
}: StepProps) {
  const [config, setConfig] = useState<StrictModeConfig>({
    strictMode: stepData?.strictMode || false,
    emergencyKey: stepData?.emergencyKey || "",
    userName: stepData?.userName || "",
  });

  const [isCapturingKey, setIsCapturingKey] = useState(false);
  const pressedKeysRef = useRef<Set<string>>(new Set());

  // Update parent component when config changes
  useEffect(() => {
    setStepData(config);
  }, [config, setStepData]);

  // Key capture logic
  useEffect(() => {
    if (!isCapturingKey) return;

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
        setConfig((prev) => ({ ...prev, emergencyKey: keyCombo }));
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
  }, [isCapturingKey]);

  const startKeyCapture = () => {
    setIsCapturingKey(true);
    pressedKeysRef.current.clear();
    setConfig((prev) => ({ ...prev, emergencyKey: "" }));
  };

  const clearKeyCapture = () => {
    setIsCapturingKey(false);
    pressedKeysRef.current.clear();
    setConfig((prev) => ({ ...prev, emergencyKey: "" }));
  };

  const handleStrictModeToggle = (enabled: boolean) => {
    setConfig((prev) => ({ ...prev, strictMode: enabled }));
  };

  const handleUserNameChange = (name: string) => {
    setConfig((prev) => ({ ...prev, userName: name }));
  };

  const canProceed = () => {
    if (!config.strictMode) return true; // Can proceed if strict mode is disabled
    return config.emergencyKey.length > 0; // Need emergency key if strict mode is enabled
  };

  return (
    <div className="py-8">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-light text-white mb-4">
          Strict mode and emergency exit
        </h1>
      </div>

      <div className="space-y-8">
        {/* User Name Input */}
        <div>
          <label className="block text-white text-lg font-medium mb-4">
            What's your name? (Optional)
          </label>
          <input
            type="text"
            value={config.userName}
            onChange={(e) => handleUserNameChange(e.target.value)}
            className="w-full p-4 bg-gray-800/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-white focus:outline-none"
            placeholder="Enter your name for personalized notifications"
          />
        </div>

        {/* Strict Mode Toggle */}
        <div>
          <label className="flex items-center space-x-4 cursor-pointer">
            <input
              type="checkbox"
              checked={config.strictMode}
              onChange={(e) => handleStrictModeToggle(e.target.checked)}
              className="w-5 h-5 text-white bg-gray-800 border-gray-600 rounded focus:ring-white focus:ring-2"
            />
            <div>
              <span className="text-white text-lg font-medium">
                Enable strict mode (block screen during focus)
              </span>
            </div>
          </label>
        </div>

        {/* Emergency Key Configuration */}
        {config.strictMode && (
          <div className="bg-yellow-900/20 border border-yellow-700/50 rounded-lg p-6">
            <h3 className="text-yellow-200 font-medium mb-4">
              Emergency Exit Configuration
            </h3>

            <div className="space-y-4">
              <div>
                <label className="block text-yellow-100 text-sm font-medium mb-2">
                  Emergency Key Combination
                </label>
                <div className="flex space-x-3">
                  <div className="flex-1">
                    <input
                      type="text"
                      value={config.emergencyKey}
                      readOnly
                      placeholder="Press your combination to exit Pausa mode"
                      className="w-full p-3 bg-gray-800/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-white focus:outline-none"
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
                  {config.emergencyKey && (
                    <button
                      onClick={clearKeyCapture}
                      className="px-4 py-3 rounded-lg font-medium bg-gray-700 text-gray-300 hover:bg-gray-600 transition-all"
                    >
                      Clear
                    </button>
                  )}
                </div>
              </div>

              <div className="bg-gray-800/30 rounded-lg p-4">
                <p className="text-gray-300 text-sm mb-2">
                  <strong>Example:</strong> ⌘ + ⇧ + P
                </p>
                <p className="text-gray-400 text-xs">
                  You can only unlock the screen using this combination. Choose
                  something you'll remember but others won't guess.
                </p>
              </div>

              {isCapturingKey && (
                <div className="bg-blue-900/20 border border-blue-700/50 rounded-lg p-4">
                  <p className="text-blue-200 text-sm">
                    🎯 Press your desired key combination now...
                  </p>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Validation Warning */}
        {config.strictMode && !config.emergencyKey && (
          <div className="bg-red-900/20 border border-red-700/50 rounded-lg p-4">
            <p className="text-red-200 text-sm">
              ⚠️ Emergency key combination is required when strict mode is
              enabled.
            </p>
          </div>
        )}
      </div>

      {/* Navigation buttons */}
      <div className="flex justify-between mt-12">
        <button
          onClick={onPrevious}
          disabled={!canGoPrevious}
          className={`px-6 py-3 rounded-lg font-medium transition-all ${
            canGoPrevious
              ? "text-gray-300 hover:text-white hover:bg-gray-800/50"
              : "text-gray-600 cursor-not-allowed"
          }`}
        >
          Previous
        </button>

        <button
          onClick={onNext}
          disabled={!canGoNext || !canProceed()}
          className={`px-8 py-3 rounded-lg font-medium transition-all ${
            canGoNext && canProceed()
              ? "bg-white text-black hover:bg-gray-200"
              : "bg-gray-600 text-gray-400 cursor-not-allowed"
          }`}
        >
          Continue
        </button>
      </div>
    </div>
  );
}
