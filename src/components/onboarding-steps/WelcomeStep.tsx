import { ArrowRightIcon } from "lucide-react";
import { StepProps } from "./types";

export default function WelcomeStep({ onNext, canGoNext }: StepProps) {
  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
      <div className="mb-6 flex flex-col items-center justify-center">
        <p className="text-4xl font-bold rounded-xl py-2 px-4 mb-4">P</p>
        <h1 className="text-5xl font-bold mb-4">Welcome to Pausa</h1>
      </div>

      <div className="text-center flex flex-col items-center justify-center">
        <p className="text-sm text-gray-400 mb-6">
          Let's set up your personalized work routine in just a few steps
        </p>

        <button
          onClick={onNext}
          disabled={!canGoNext}
          className="bg-white text-black px-6 py-3 rounded-lg font-medium hover:bg-gray-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          Start Setup
          <ArrowRightIcon className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
