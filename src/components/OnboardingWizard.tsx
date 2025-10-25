import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ArrowLeftIcon, ArrowRightIcon } from "lucide-react";

// Types for onboarding
export type OnboardingStep = "Welcome" | "WorkSchedule" | "Complete";

interface OnboardingWizardProps {
  onComplete?: () => void;
  onSkip?: () => void;
}

interface StepProps {
  onNext: () => void;
  onPrevious: () => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
}

// Welcome Step Component
function WelcomeStep({ onNext, canGoNext }: StepProps) {
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

// Placeholder for WorkSchedule step (will be implemented in later tasks)
function WorkScheduleStep({
  onNext,
  onPrevious,
  canGoNext,
  canGoPrevious,
}: StepProps) {
  return (
    <div className="flex flex-col items-center justify-center min-h-[400px] text-center">
      <h2 className="text-3xl font-bold mb-4">Work Schedule</h2>
      <p className="text-gray-300 mb-8">
        This step will be implemented in task 2.1
      </p>

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
          onClick={onNext}
          disabled={!canGoNext}
          className="bg-white text-black px-6 py-3 rounded-lg font-medium hover:bg-gray-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
        >
          Next
          <ArrowRightIcon className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}

// Complete step placeholder
function CompleteStep({ onPrevious, canGoPrevious }: StepProps) {
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

export default function OnboardingWizard({
  onComplete,
  onSkip,
}: OnboardingWizardProps) {
  const [currentStep, setCurrentStep] = useState<OnboardingStep>("Welcome");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Initialize onboarding when component mounts
  useEffect(() => {
    initializeOnboarding();
  }, []);

  const initializeOnboarding = async () => {
    setIsLoading(true);
    setError(null);

    try {
      const step = await invoke<OnboardingStep>("start_onboarding");
      setCurrentStep(step);
    } catch (err) {
      console.error("Failed to initialize onboarding:", err);
      setError("Failed to start onboarding. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  const handleNext = async () => {
    setIsLoading(true);
    setError(null);

    try {
      // For now, we'll handle navigation locally since next_onboarding_step
      // will be implemented in task 1.3
      switch (currentStep) {
        case "Welcome":
          setCurrentStep("WorkSchedule");
          break;
        case "WorkSchedule":
          setCurrentStep("Complete");
          break;
        case "Complete":
          onComplete?.();
          break;
      }
    } catch (err) {
      console.error("Failed to navigate to next step:", err);
      setError("Failed to proceed. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  const handlePrevious = async () => {
    setIsLoading(true);
    setError(null);

    try {
      // For now, we'll handle navigation locally since previous_onboarding_step
      // will be implemented in task 1.3
      switch (currentStep) {
        case "WorkSchedule":
          setCurrentStep("Welcome");
          break;
        case "Complete":
          setCurrentStep("WorkSchedule");
          break;
        case "Welcome":
          // Cannot go back from welcome
          break;
      }
    } catch (err) {
      console.error("Failed to navigate to previous step:", err);
      setError("Failed to go back. Please try again.");
    } finally {
      setIsLoading(false);
    }
  };

  const canGoNext = () => {
    return currentStep !== "Complete" && !isLoading;
  };

  const canGoPrevious = () => {
    return currentStep !== "Welcome" && !isLoading;
  };

  const renderCurrentStep = () => {
    const stepProps: StepProps = {
      onNext: handleNext,
      onPrevious: handlePrevious,
      canGoNext: canGoNext(),
      canGoPrevious: canGoPrevious(),
    };

    switch (currentStep) {
      case "Welcome":
        return <WelcomeStep {...stepProps} />;
      case "WorkSchedule":
        return <WorkScheduleStep {...stepProps} />;
      case "Complete":
        return <CompleteStep {...stepProps} />;
      default:
        return <WelcomeStep {...stepProps} />;
    }
  };

  if (isLoading && !currentStep) {
    return (
      <div className="min-h-screen bg-gradient-to-br from-zinc-900 via-zinc-800 to-zinc-900 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-white mx-auto mb-4"></div>
          <p className="text-gray-300">Loading onboarding...</p>
        </div>
      </div>
    );
  }

  return (
    <div
      className="min-h-screen bg-gradient-to-br from-zinc-900 via-zinc-800 to-zinc-900 flex items-center justify-center  relative overflow-hidden"
      style={{
        fontFamily: "BBH Sans Bartle",
        fontWeight: 400,
        fontStyle: "normal",
      }}
    >
      {/* Background decorative elements */}
      <div className="absolute top-0 right-0 w-[600px] h-[600px] bg-white/10 rounded-full blur-3xl translate-x-1/2 -translate-y-1/2" />
      <div className="absolute bottom-0 left-0 w-[600px] h-[600px] bg-white/10 rounded-full blur-3xl -translate-x-1/2 translate-y-1/2" />

      <div className="w-full max-w-2xl relative z-10">
        {/* Progress indicator */}
        <div className="mb-8">
          <div className="flex justify-center items-center space-x-2 mb-4">
            <div
              className={`w-2 h-2 rounded-full ${
                currentStep === "Welcome" ? "bg-white" : "bg-gray-600"
              }`}
            />
            <div
              className={`w-2 h-2 rounded-full ${
                currentStep === "WorkSchedule" ? "bg-white" : "bg-gray-600"
              }`}
            />
            <div
              className={`w-2 h-2 rounded-full ${
                currentStep === "Complete" ? "bg-white" : "bg-gray-600"
              }`}
            />
          </div>
          <p className="text-center text-sm text-gray-400">
            Step{" "}
            {currentStep === "Welcome"
              ? "1"
              : currentStep === "WorkSchedule"
              ? "2"
              : "3"}{" "}
            of 3
          </p>
        </div>

        {/* Error display */}
        {error && (
          <div className="mb-6 p-4 bg-red-900/50 border border-red-700 rounded-lg text-red-200 text-center">
            {error}
            <button
              onClick={initializeOnboarding}
              className="ml-2 underline hover:no-underline"
            >
              Retry
            </button>
          </div>
        )}

        {/* Current step content */}
        <div className="bg-black/20 backdrop-blur-sm rounded-xl px-8 border border-white/10">
          {renderCurrentStep()}
        </div>

        {/*    {onSkip && (
          <div className="text-center mt-6">
            <button
              onClick={onSkip}
              className="text-sm text-gray-400 hover:text-gray-300 transition-colors"
            >
              Skip setup for now
            </button>
          </div>
        )} */}
      </div>
    </div>
  );
}
