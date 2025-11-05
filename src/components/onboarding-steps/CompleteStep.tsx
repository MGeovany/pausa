import { ArrowLeftIcon } from "lucide-react";
import { StepProps } from "./types";

export default function CompleteStep({ onPrevious, canGoPrevious }: StepProps) {
  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
      <h2 className="text-3xl font-bold mb-4">Setup Complete</h2>
      <p className="text-gray-300 mb-8">Your onboarding is complete!</p>

      <button
        onClick={onPrevious}
        disabled={!canGoPrevious}
        className="bg-gray-600 text-white px-6 py-3 rounded-lg font-medium hover:bg-gray-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
      >
        <ArrowLeftIcon className="w-4 h-4" />
        Previous
      </button>
    </div>
  );
}
