import { useState } from "react";
import { ArrowLeftIcon, ArrowRightIcon } from "lucide-react";
import { StepProps } from "./types";

export default function WorkScheduleStep({
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
  stepData,
  setStepData,
}: StepProps) {
  const [useWorkSchedule, setUseWorkSchedule] = useState<boolean | null>(
    stepData?.useWorkSchedule ?? null
  );

  const handleNext = () => {
    if (useWorkSchedule !== null) {
      // Store the choice in step data
      setStepData({ useWorkSchedule });
      onNext();
    }
  };

  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] text-center p-4">
      <h2 className="text-3xl font-bold mb-4">Work Schedule</h2>
      <p
        className="text-gray-300 tracking-wider mb-4"
        style={{ fontFamily: "Cal Sans" }}
      >
        Do you want Pausa to organize your blocks according to your work
        schedule?
      </p>

      <div className="flex flex-col gap-4 mb-8 w-full max-w-md">
        <button
          onClick={() => setUseWorkSchedule(true)}
          className={`p-4 rounded-lg border-2 transition-colors ${
            useWorkSchedule === true
              ? "border-white bg-white/10 text-white"
              : "border-gray-600 text-gray-300 hover:border-gray-500"
          }`}
        >
          <div className="text-left">
            <div className="font-medium">Yes, use my work schedule</div>
            <div className="text-sm text-gray-400">Configure work hours</div>
          </div>
        </button>

        <button
          onClick={() => setUseWorkSchedule(false)}
          className={`p-4 rounded-lg border-2 transition-colors ${
            useWorkSchedule === false
              ? "border-white bg-white/10 text-white"
              : "border-gray-600 text-gray-300 hover:border-gray-500"
          }`}
        >
          <div className="text-left">
            <div className="font-medium">No, configure manually</div>
            <div className="text-sm text-gray-400">Skip work hours setup</div>
          </div>
        </button>
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
          disabled={!canGoNext || useWorkSchedule === null}
          className="bg-white text-black px-6 py-3 rounded-lg font-medium hover:bg-gray-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          Continue
          <ArrowRightIcon className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}

