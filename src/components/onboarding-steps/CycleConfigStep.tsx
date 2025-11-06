import { useState, useEffect } from "react";
import type { StepProps } from "./types";

interface CycleConfig {
  focusDuration: number; // minutes
  breakDuration: number; // minutes
  longBreakDuration: number; // minutes
  cyclesPerLongBreak: number;
}

const FOCUS_OPTIONS = [
  { value: 25, label: "25 min" },
  { value: 30, label: "30 min" },
  { value: 45, label: "45 min" },
  { value: 0, label: "Custom" },
];

const BREAK_OPTIONS = [
  { value: 5, label: "5 min" },
  { value: 10, label: "10 min" },
  { value: 15, label: "15 min" },
];

const LONG_BREAK_CYCLES = [
  { value: 3, label: "3 cycles" },
  { value: 4, label: "4 cycles" },
];

const LONG_BREAK_DURATION = [
  { value: 15, label: "15 min" },
  { value: 20, label: "20 min" },
];

export default function CycleConfigStep({
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
  stepData,
  setStepData,
}: StepProps) {
  const [config, setConfig] = useState<CycleConfig>({
    focusDuration: stepData?.focusDuration || 25,
    breakDuration: stepData?.breakDuration || 5,
    longBreakDuration: stepData?.longBreakDuration || 15,
    cyclesPerLongBreak: stepData?.cyclesPerLongBreak || 4,
  });

  const [customFocusDuration, setCustomFocusDuration] = useState<string>(
    stepData?.customFocusDuration || "25"
  );

  const [showCustomInput, setShowCustomInput] = useState(
    stepData?.focusDuration &&
      !FOCUS_OPTIONS.slice(0, 3).some(
        (opt) => opt.value === stepData.focusDuration
      )
  );

  // Update parent component when config changes
  useEffect(() => {
    const finalConfig = {
      ...config,
      customFocusDuration,
      focusDuration: showCustomInput
        ? parseInt(customFocusDuration) || 25
        : config.focusDuration,
    };
    setStepData(finalConfig);
  }, [config, customFocusDuration, showCustomInput, setStepData]);

  const handleFocusOptionChange = (value: number) => {
    if (value === 0) {
      setShowCustomInput(true);
      setConfig((prev) => ({
        ...prev,
        focusDuration: parseInt(customFocusDuration) || 25,
      }));
    } else {
      setShowCustomInput(false);
      setConfig((prev) => ({ ...prev, focusDuration: value }));
    }
  };

  const handleCustomFocusChange = (value: string) => {
    setCustomFocusDuration(value);
    const numValue = parseInt(value) || 25;
    if (showCustomInput) {
      setConfig((prev) => ({ ...prev, focusDuration: numValue }));
    }
  };

  const getPreviewText = () => {
    const focus = showCustomInput
      ? parseInt(customFocusDuration) || 25
      : config.focusDuration;
    return `${focus} min focus / ${config.breakDuration} min break / ${config.cyclesPerLongBreak} cycles / ${config.longBreakDuration} min long break`;
  };

  return (
    <div className="py-8">
      <div className="text-center mb-8">
        <h1 className="text-3xl font-light text-white mb-4">
          Configure your focus cycles
        </h1>
        <p className="text-gray-300 text-lg">
          Customize the duration of your focus and break periods
        </p>
      </div>

      <div className="space-y-8">
        {/* Focus Duration */}
        <div>
          <label className="block text-white text-lg font-medium mb-4">
            Focus Duration
          </label>
          <div className="grid grid-cols-2 gap-3 mb-4">
            {FOCUS_OPTIONS.map((option) => (
              <button
                key={option.value}
                onClick={() => handleFocusOptionChange(option.value)}
                className={`p-4 rounded-lg border-2 transition-all ${
                  (option.value === 0 && showCustomInput) ||
                  (option.value !== 0 &&
                    !showCustomInput &&
                    config.focusDuration === option.value)
                    ? "border-white bg-white/10 text-white"
                    : "border-gray-600 bg-gray-800/50 text-gray-300 hover:border-gray-500"
                }`}
              >
                {option.label}
              </button>
            ))}
          </div>

          {showCustomInput && (
            <div className="mt-4">
              <input
                type="number"
                min="1"
                max="120"
                value={customFocusDuration}
                onChange={(e) => handleCustomFocusChange(e.target.value)}
                className="w-full p-3 bg-gray-800/50 border border-gray-600 rounded-lg text-white placeholder-gray-400 focus:border-white focus:outline-none"
                placeholder="Enter minutes (1-120)"
              />
            </div>
          )}
        </div>

        {/* Break Duration */}
        <div>
          <label className="block text-white text-lg font-medium mb-4">
            Break Duration
          </label>
          <div className="grid grid-cols-3 gap-3">
            {BREAK_OPTIONS.map((option) => (
              <button
                key={option.value}
                onClick={() =>
                  setConfig((prev) => ({
                    ...prev,
                    breakDuration: option.value,
                  }))
                }
                className={`p-4 rounded-lg border-2 transition-all ${
                  config.breakDuration === option.value
                    ? "border-white bg-white/10 text-white"
                    : "border-gray-600 bg-gray-800/50 text-gray-300 hover:border-gray-500"
                }`}
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>

        {/* Long Break Cycles */}
        <div>
          <label className="block text-white text-lg font-medium mb-4">
            Long break after
          </label>
          <div className="grid grid-cols-2 gap-3">
            {LONG_BREAK_CYCLES.map((option) => (
              <button
                key={option.value}
                onClick={() =>
                  setConfig((prev) => ({
                    ...prev,
                    cyclesPerLongBreak: option.value,
                  }))
                }
                className={`p-4 rounded-lg border-2 transition-all ${
                  config.cyclesPerLongBreak === option.value
                    ? "border-white bg-white/10 text-white"
                    : "border-gray-600 bg-gray-800/50 text-gray-300 hover:border-gray-500"
                }`}
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>

        {/* Long Break Duration */}
        <div>
          <label className="block text-white text-lg font-medium mb-4">
            Long Break Duration
          </label>
          <div className="grid grid-cols-2 gap-3">
            {LONG_BREAK_DURATION.map((option) => (
              <button
                key={option.value}
                onClick={() =>
                  setConfig((prev) => ({
                    ...prev,
                    longBreakDuration: option.value,
                  }))
                }
                className={`p-4 rounded-lg border-2 transition-all ${
                  config.longBreakDuration === option.value
                    ? "border-white bg-white/10 text-white"
                    : "border-gray-600 bg-gray-800/50 text-gray-300 hover:border-gray-500"
                }`}
              >
                {option.label}
              </button>
            ))}
          </div>
        </div>

        {/* Configuration Preview */}
        <div className="bg-blue-900/20 border border-blue-700/50 rounded-lg p-4">
          <h3 className="text-blue-200 font-medium mb-2">
            Configuration Preview
          </h3>
          <p className="text-blue-100 text-lg">{getPreviewText()}</p>
        </div>
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
          disabled={!canGoNext}
          className={`px-8 py-3 rounded-lg font-medium transition-all ${
            canGoNext
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
