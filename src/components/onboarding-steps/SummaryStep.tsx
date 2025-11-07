import { useState, useEffect } from "react";
import { ArrowLeftIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { StepProps } from "./types";

interface OnboardingConfig {
  useWorkSchedule: boolean;
  workHours?: {
    startTime: string;
    endTime: string;
    timezone: string;
  };
  focusDuration: number;
  breakDuration: number;
  longBreakDuration: number;
  cyclesPerLongBreak: number;
  strictMode: boolean;
  emergencyKey?: string;
  userName?: string;
}

export default function SummaryStep({
  onNext,
  onPrevious,
  canGoPrevious,
  setStepData,
}: StepProps) {
  const [config, setConfig] = useState<OnboardingConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isCompleting, setIsCompleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadOnboardingConfig();
  }, []);

  const loadOnboardingConfig = async () => {
    setIsLoading(true);
    setError(null);

    try {
      // Get all configuration from backend
      const [userSettings, workSchedule] = await Promise.all([
        invoke("get_user_settings"),
        invoke("get_work_schedule"),
      ]);

      const configData: OnboardingConfig = {
        useWorkSchedule: (workSchedule as any)?.use_work_schedule || false,
        workHours: (workSchedule as any)?.use_work_schedule
          ? {
              startTime: (workSchedule as any)?.work_start_time || "09:00",
              endTime: (workSchedule as any)?.work_end_time || "18:00",
              timezone: (workSchedule as any)?.timezone || "local",
            }
          : undefined,
        focusDuration: Math.floor(
          ((userSettings as any)?.focus_duration || 1500) / 60
        ),
        breakDuration: Math.floor(
          ((userSettings as any)?.short_break_duration || 300) / 60
        ),
        longBreakDuration: Math.floor(
          ((userSettings as any)?.long_break_duration || 900) / 60
        ),
        cyclesPerLongBreak:
          (userSettings as any)?.cycles_per_long_break_v2 || 4,
        strictMode: (userSettings as any)?.strict_mode || false,
        emergencyKey: (userSettings as any)?.emergency_key_combination,
        userName: (userSettings as any)?.user_name,
      };

      setConfig(configData);
      setStepData(configData);
    } catch (err) {
      console.error("Failed to load onboarding config:", err);
      setError("Failed to load configuration. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  const handleCompleteOnboarding = async () => {
    if (!config) return;

    setIsCompleting(true);
    setError(null);

    try {
      // Use the onNext handler to let the wizard handle completion
      await onNext();
    } catch (err) {
      console.error("Failed to complete onboarding:", err);
      setError("Failed to complete onboarding. Please try again.");
    } finally {
      setIsCompleting(false);
    }
  };

  const formatTime = (time: string) => {
    const [hours, minutes] = time.split(":");
    const hour = parseInt(hours);
    const ampm = hour >= 12 ? "PM" : "AM";
    const displayHour = hour === 0 ? 12 : hour > 12 ? hour - 12 : hour;
    return `${displayHour}:${minutes} ${ampm}`;
  };

  if (isLoading) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white mx-auto mb-4"></div>
        <p className="text-gray-300">Loading configuration...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
        <div className="mb-6 p-4 bg-red-900/50 border border-red-700 rounded-lg text-red-200">
          {error}
        </div>
        <button
          onClick={loadOnboardingConfig}
          className="bg-blue-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-blue-700 transition-colors"
        >
          Retry
        </button>
      </div>
    );
  }

  if (!config) {
    return (
      <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
        <p className="text-gray-300">No configuration found.</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] text-center py-8">
      {/* Header */}
      <div className="mb-8">
        <h2 className="text-3xl font-bold mb-2">All set</h2>
        <p className="text-gray-300 text-lg">
          Perfect, your routine is configured
        </p>
      </div>

      {/* Configuration Summary */}
      <div className="bg-white/5 rounded-lg p-6 mb-8 max-w-md w-full">
        <div className="text-center mb-4">
          <div className="text-2xl font-bold text-white">
            {config.focusDuration} min focus / {config.breakDuration} min break
            / {config.cyclesPerLongBreak} cycles / {config.longBreakDuration}{" "}
            min long break
          </div>
        </div>

        <div className="space-y-3 text-left">
          {config.useWorkSchedule && config.workHours && (
            <div className="flex justify-between items-center">
              <span className="text-gray-400">Work Hours:</span>
              <span className="text-white">
                {formatTime(config.workHours.startTime)} -{" "}
                {formatTime(config.workHours.endTime)}
              </span>
            </div>
          )}

          <div className="flex justify-between items-center">
            <span className="text-gray-400">Strict Mode:</span>
            <span className="text-white">
              {config.strictMode ? "Enabled" : "Disabled"}
            </span>
          </div>

          {config.strictMode && config.emergencyKey && (
            <div className="flex justify-between items-center">
              <span className="text-gray-400">Emergency Key:</span>
              <span className="text-white font-mono text-sm">
                {config.emergencyKey}
              </span>
            </div>
          )}

          {config.userName && (
            <div className="flex justify-between items-center">
              <span className="text-gray-400">Name:</span>
              <span className="text-white">{config.userName}</span>
            </div>
          )}
        </div>
      </div>

      {/* Action Buttons */}
      <div className="flex gap-4">
        <button
          onClick={onPrevious}
          disabled={!canGoPrevious || isCompleting}
          className="bg-gray-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          <ArrowLeftIcon className="w-4 h-4" />
          Previous
        </button>

        <button
          onClick={handleCompleteOnboarding}
          disabled={isCompleting}
          className="bg-white text-black px-8 py-3 rounded-lg font-medium hover:bg-gray-200 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          {isCompleting ? (
            <>
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
              Completing...
            </>
          ) : (
            "Start Pausa"
          )}
        </button>
      </div>
    </div>
  );
}
