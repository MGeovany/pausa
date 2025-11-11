import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import WelcomeStep from "./onboarding-steps/WelcomeStep";
import WorkHoursStep from "./onboarding-steps/WorkHoursStep";
import CycleConfigStep from "./onboarding-steps/CycleConfigStep";
import StrictModeStep from "./onboarding-steps/StrictModeStep";
import SummaryStep from "./onboarding-steps/SummaryStep";
import CompleteStep from "./onboarding-steps/CompleteStep";
import type { StepProps } from "./onboarding-steps/types";

// Types for onboarding
export type OnboardingStep =
  | "Welcome"
  | "WorkHours"
  | "CycleConfig"
  | "StrictMode"
  | "Summary"
  | "Complete";

interface OnboardingWizardProps {
  onComplete?: (config: any) => void;
  onSkip?: () => void;
}

export default function OnboardingWizard({
  onComplete: _onComplete,
  onSkip: _onSkip,
}: OnboardingWizardProps) {
  const [currentStep, setCurrentStep] = useState<OnboardingStep>("Welcome");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationErrors, setValidationErrors] = useState<string[]>([]);
  const [isRecovering, setIsRecovering] = useState(false);

  // Initialize onboarding when component mounts
  useEffect(() => {
    initializeOnboarding();
  }, []);

  const initializeOnboarding = async () => {
    setIsLoading(true);
    setError(null);
    setValidationErrors([]);

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

  const validateCurrentStep = async (stepData: any): Promise<boolean> => {
    try {
      await invoke("validate_step_config", {
        step: currentStep,
        stepData: stepData || {},
      });
      setValidationErrors([]);
      return true;
    } catch (err) {
      console.error("Step validation failed:", err);
      if (typeof err === "string" && err.includes("validation errors:")) {
        const errors = err.replace("Step validation errors: ", "").split("; ");
        setValidationErrors(errors);
      } else {
        setValidationErrors([typeof err === "string" ? err : "Validation failed"]);
      }
      return false;
    }
  };

  const createBackup = async (): Promise<void> => {
    try {
      const backupId = await invoke<string>("create_configuration_backup", {
        backupType: "pre_update",
        description: `Backup before ${currentStep} step`,
      });
      console.log("✅ [Frontend] Backup created:", backupId);
    } catch (err) {
      console.warn("⚠️ [Frontend] Failed to create backup:", err);
      // Don't block the flow for backup failures
    }
  };

  const recoverFromError = async () => {
    setIsRecovering(true);
    setError(null);
    setValidationErrors([]);

    try {
      // Try to get configuration health check
      const healthCheck = await invoke("get_configuration_health_check");
      console.log("🏥 [Frontend] Health check:", healthCheck);

      // Reset onboarding if needed
      await invoke("reset_onboarding_for_testing");
      
      // Reinitialize
      await initializeOnboarding();
    } catch (err) {
      console.error("❌ [Frontend] Recovery failed:", err);
      setError("Recovery failed. Please restart the application.");
    } finally {
      setIsRecovering(false);
    }
  };

  const [stepData, setStepData] = useState<Record<string, any>>({});

  const handleNext = async () => {
    setIsLoading(true);
    setError(null);
    setValidationErrors([]);

    try {
      // Get step data for current step
      const currentStepData = stepData[currentStep] || null;

      // Validate current step data before proceeding
      if (currentStepData && !(await validateCurrentStep(currentStepData))) {
        setIsLoading(false);
        return;
      }

      // Create backup before making changes
      await createBackup();
      console.log(`🔄 [Frontend] Attempting to navigate from ${currentStep}`);

      // Save work schedule data if we're leaving the WorkHours step
      if (currentStep === "WorkHours" && currentStepData) {
        try {
          await invoke("save_work_schedule", {
            config: {
              use_work_schedule: true,
              work_start_time: currentStepData.startTime,
              work_end_time: currentStepData.endTime,
              timezone: currentStepData.timezone,
            },
          });
          console.log("✅ [Frontend] Work schedule saved successfully");
        } catch (saveErr) {
          console.error("❌ [Frontend] Failed to save work schedule:", saveErr);
          setError("Failed to save work schedule. Please try again.");
          return;
        }
      }



      // Save cycle configuration if we're leaving the CycleConfig step
      if (currentStep === "CycleConfig" && currentStepData) {
        try {
          await invoke("save_cycle_config", {
            config: {
              focus_duration: currentStepData.focusDuration,
              break_duration: currentStepData.breakDuration,
              long_break_duration: currentStepData.longBreakDuration,
              cycles_per_long_break: currentStepData.cyclesPerLongBreak,
            },
          });
          console.log("✅ [Frontend] Cycle configuration saved successfully");
        } catch (saveErr) {
          console.error("❌ [Frontend] Failed to save cycle config:", saveErr);
          setError("Failed to save cycle configuration. Please try again.");
          return;
        }
      }

      // Save strict mode configuration if we're leaving the StrictMode step
      if (currentStep === "StrictMode" && currentStepData) {
        try {
          // Save user name if provided
          if (currentStepData.userName) {
            await invoke("update_user_name", {
              userName: currentStepData.userName,
            });
          }

          // Save strict mode and emergency key
          await invoke("save_strict_mode_config", {
            config: {
              strict_mode: currentStepData.strictMode,
              emergency_key_combination: currentStepData.emergencyKey || null,
            },
          });
          console.log(
            "✅ [Frontend] Strict mode configuration saved successfully"
          );
        } catch (saveErr) {
          console.error(
            "❌ [Frontend] Failed to save strict mode config:",
            saveErr
          );
          setError(
            "Failed to save strict mode configuration. Please try again."
          );
          return;
        }
      }

      const nextStep = await invoke<OnboardingStep>("next_onboarding_step", {
        stepData: currentStepData,
      });

      console.log(`✅ [Frontend] Successfully navigated to ${nextStep}`);
      setCurrentStep(nextStep);

      // Handle completion
      if (nextStep === "Complete") {
        console.log("🎉 [Frontend] Onboarding completed");
        
        // Collect all configuration data
        const finalConfig = {
          // Work schedule configuration - always enabled now
          workSchedule: {
            useWorkSchedule: true,
            workStartTime: stepData.WorkHours?.startTime || null,
            workEndTime: stepData.WorkHours?.endTime || null,
          },
          // Cycle configuration
          focusDuration: stepData.CycleConfig?.focusDuration || 25,
          breakDuration: stepData.CycleConfig?.breakDuration || 5,
          longBreakDuration: stepData.CycleConfig?.longBreakDuration || 15,
          cyclesPerLongBreak: stepData.CycleConfig?.cyclesPerLongBreak || 4,
          // Strict mode configuration
          strictMode: stepData.StrictMode?.strictMode || false,
          emergencyKey: stepData.StrictMode?.emergencyKey || null,
          userName: stepData.StrictMode?.userName || null,
        };

        console.log("📋 [Frontend] Final onboarding configuration:", finalConfig);

        // Validate final configuration before completion
        try {
          await invoke("validate_onboarding_config", { config: finalConfig });
          console.log("✅ [Frontend] Final configuration validation passed");
        } catch (validationErr) {
          console.error("❌ [Frontend] Final configuration validation failed:", validationErr);
          setError(`Configuration validation failed: ${validationErr}`);
          return;
        }

        // Complete onboarding in backend
        try {
          await invoke("complete_onboarding", { finalConfig });
          console.log("✅ [Frontend] Onboarding completion saved to backend");
        } catch (completeErr) {
          console.error("❌ [Frontend] Failed to complete onboarding:", completeErr);
          setError("Failed to complete onboarding. Please try again.");
          return;
        }

        // Onboarding is now complete, the app should transition to main view
        if (_onComplete) {
          _onComplete(finalConfig);
        }
      }
    } catch (err) {
      console.error("❌ [Frontend] Navigation failed:", err);

      // Provide user-friendly error messages
      let errorMessage = "Failed to proceed. Please try again.";
      if (typeof err === "string") {
        if (err.includes("Cannot proceed beyond completion")) {
          errorMessage = "You've already completed the onboarding.";
        } else if (err.includes("Navigation failed")) {
          errorMessage =
            "Unable to move to the next step. Please check your input.";
        }
      }

      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const handlePrevious = async () => {
    setIsLoading(true);
    setError(null);

    try {
      console.log(
        `🔙 [Frontend] Attempting to navigate back from ${currentStep}`
      );

      const previousStep = await invoke<OnboardingStep>(
        "previous_onboarding_step"
      );

      console.log(
        `✅ [Frontend] Successfully navigated back to ${previousStep}`
      );
      setCurrentStep(previousStep);
    } catch (err) {
      console.error("❌ [Frontend] Backward navigation failed:", err);

      // Provide user-friendly error messages
      let errorMessage = "Failed to go back. Please try again.";
      if (typeof err === "string") {
        if (err.includes("Cannot go back from welcome")) {
          errorMessage = "You're already at the first step.";
        } else if (err.includes("Backward navigation failed")) {
          errorMessage = "Unable to go back to the previous step.";
        }
      }

      setError(errorMessage);
    } finally {
      setIsLoading(false);
    }
  };

  const canGoNext = () => {
    return currentStep !== "Complete" && currentStep !== "Summary" && !isLoading;
  };

  const canGoPrevious = () => {
    return currentStep !== "Welcome" && !isLoading;
  };

  const renderCurrentStep = () => {
    const currentStepData = stepData[currentStep] || {};

    const updateStepData = (data: any) => {
      setStepData((prev) => ({
        ...prev,
        [currentStep]: data,
      }));
    };

    const stepProps: StepProps = {
      onNext: handleNext,
      onPrevious: handlePrevious,
      canGoNext: canGoNext(),
      canGoPrevious: canGoPrevious(),
      stepData: currentStepData,
      setStepData: updateStepData,
    };

    switch (currentStep) {
      case "Welcome":
        return <WelcomeStep {...stepProps} />;
      case "WorkHours":
        return <WorkHoursStep {...stepProps} />;
      case "CycleConfig":
        return <CycleConfigStep {...stepProps} />;
      case "StrictMode":
        return <StrictModeStep {...stepProps} />;
      case "Summary":
        return <SummaryStep {...stepProps} />;
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

      <div className="w-full max-w-2xl relative z-10 p-4">
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
                currentStep === "WorkHours" ? "bg-white" : "bg-gray-600"
              }`}
            />
            <div
              className={`w-2 h-2 rounded-full ${
                currentStep === "CycleConfig" ? "bg-white" : "bg-gray-600"
              }`}
            />
            <div
              className={`w-2 h-2 rounded-full ${
                currentStep === "StrictMode" ? "bg-white" : "bg-gray-600"
              }`}
            />
            <div
              className={`w-2 h-2 rounded-full ${
                currentStep === "Summary" ? "bg-white" : "bg-gray-600"
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
              : currentStep === "WorkHours"
              ? "2"
              : currentStep === "CycleConfig"
              ? "3"
              : currentStep === "StrictMode"
              ? "4"
              : currentStep === "Summary"
              ? "5"
              : "6"}{" "}
            of 6
          </p>
        </div>

        {/* Error display */}
        {error && (
          <div className="mb-6 p-4 bg-red-900/50 border border-red-700 rounded-lg text-red-200">
            <div className="text-center mb-2">{error}</div>
            <div className="flex justify-center space-x-2">
              <button
                onClick={initializeOnboarding}
                className="px-3 py-1 text-sm underline hover:no-underline"
                disabled={isRecovering}
              >
                Retry
              </button>
              <button
                onClick={recoverFromError}
                className="px-3 py-1 text-sm bg-red-700 hover:bg-red-600 rounded transition-colors"
                disabled={isRecovering}
              >
                {isRecovering ? "Recovering..." : "Recover"}
              </button>
            </div>
          </div>
        )}

        {/* Validation errors display */}
        {validationErrors.length > 0 && (
          <div className="mb-6 p-4 bg-yellow-900/50 border border-yellow-700 rounded-lg text-yellow-200">
            <div className="font-medium mb-2">Please fix the following issues:</div>
            <ul className="list-disc list-inside space-y-1 text-sm">
              {validationErrors.map((error, index) => (
                <li key={index}>{error}</li>
              ))}
            </ul>
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
