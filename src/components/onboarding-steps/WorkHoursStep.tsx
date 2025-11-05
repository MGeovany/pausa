import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeftIcon, ArrowRightIcon } from "lucide-react";
import { StepProps } from "./types";

export default function WorkHoursStep({
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
  stepData,
  setStepData,
}: StepProps) {
  const [startTime, setStartTime] = useState(stepData?.startTime || "09:00");
  const [endTime, setEndTime] = useState(stepData?.endTime || "18:00");
  const [error, setError] = useState<string | null>(null);
  const [systemTimezone, setSystemTimezone] = useState<string>("");

  // Get system timezone on mount
  useEffect(() => {
    const getTimezone = async () => {
      try {
        const timezone = await invoke<string>("get_system_timezone_info");
        setSystemTimezone(timezone);
      } catch (err) {
        console.error("Failed to get system timezone:", err);
        setSystemTimezone(Intl.DateTimeFormat().resolvedOptions().timeZone);
      }
    };
    getTimezone();
  }, []);

  const validateTimes = async (
    start: string,
    end: string
  ): Promise<boolean> => {
    try {
      await invoke("validate_work_hours", {
        startTime: start,
        endTime: end,
      });
      setError(null);
      return true;
    } catch (err) {
      const errorMessage = typeof err === "string" ? err : "Invalid time range";
      setError(errorMessage);
      return false;
    }
  };

  const handleStartTimeChange = (time: string) => {
    setStartTime(time);
    validateTimes(time, endTime);
  };

  const handleEndTimeChange = (time: string) => {
    setEndTime(time);
    validateTimes(startTime, time);
  };

  const handleNext = async () => {
    const isValid = await validateTimes(startTime, endTime);
    if (isValid) {
      // Store the work hours data
      setStepData({
        startTime,
        endTime,
        timezone:
          systemTimezone || Intl.DateTimeFormat().resolvedOptions().timeZone,
      });
      onNext();
    }
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] text-center p-4">
      <h2 className="text-3xl font-bold mb-4">Work Hours</h2>
      <p className="text-gray-300 mb-2">What hours do you normally work?</p>
      <p className="text-sm text-gray-400 mb-8">
        This will help schedule your focus and break cycles
      </p>

      <div className="flex flex-col gap-6 mb-8 w-full max-w-md">
        <div className="flex items-center gap-4">
          <div className="flex-1">
            <label className="block text-sm text-gray-400 mb-2">
              Start Time
            </label>
            <input
              type="time"
              value={startTime}
              onChange={(e) => handleStartTimeChange(e.target.value)}
              className="w-full p-3 bg-transparent border border-gray-600 rounded-lg text-white focus:border-white focus:outline-none focus:ring-0 focus-visible:outline-none"
            />
          </div>

          <div className="flex-1">
            <label className="block text-sm text-gray-400 mb-2">End Time</label>
            <input
              type="time"
              value={endTime}
              onChange={(e) => handleEndTimeChange(e.target.value)}
              className="w-full p-3 bg-transparent border border-gray-600 rounded-lg text-white focus:border-white focus:outline-none focus:ring-0 focus-visible:outline-none"
            />
          </div>
        </div>

        {error && <div className="text-red-400 text-sm">{error}</div>}
      </div>

      <div className="flex gap-4">
        <button
          onClick={onPrevious}
          disabled={!canGoPrevious}
          className="bg-gray-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          <ArrowLeftIcon className="w-4 h-4" />
          Previous
        </button>

        <button
          onClick={handleNext}
          disabled={!canGoNext || error !== null}
          className="bg-white text-black px-6 py-3 rounded-lg font-medium hover:bg-gray-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          Continue
          <ArrowRightIcon className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
