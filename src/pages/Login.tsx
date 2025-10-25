import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { ArrowRightIcon } from "lucide-react";
import OnboardingWizard from "../components/OnboardingWizard";

export default function Login() {
  const [status, setStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [showOnboarding, setShowOnboarding] = useState(true);
  const [isOnboardingComplete, setIsOnboardingComplete] = useState(false);

  console.log(
    "show onboarding",
    showOnboarding,
    "onboarding complete",
    isOnboardingComplete
  );

  useEffect(() => {
    const unlistenSuccess = listen("auth:success", (event) => {
      setStatus("Login successful! Redirecting...");
      setIsLoading(false);
      setTimeout(() => {
        window.location.hash = "#/dashboard";
      }, 1000);
    });

    const unlistenError = listen("auth:error", (e) => {
      setStatus("Error: " + String(e.payload));
      setIsLoading(false);
    });

    // Check if tokens exist and onboarding status
    const checkTokens = async () => {
      try {
        const tokens = await invoke("read_tokens");
        if (tokens) {
          // Check if onboarding is complete
          try {
            const onboardingComplete = await invoke<boolean>(
              "is_onboarding_complete"
            );
            setIsOnboardingComplete(onboardingComplete);

            if (onboardingComplete) {
              window.location.hash = "#/dashboard";
            } else {
              setShowOnboarding(true);
            }
          } catch (error) {
            console.error("Error checking onboarding status:", error);
            // If we can't check onboarding status, assume it's not complete
            setShowOnboarding(true);
          }
        }
      } catch (error) {
        console.error("Error checking tokens:", error);
      }
    };

    // Check tokens every 2 seconds
    const interval = setInterval(checkTokens, 2000);

    // Cleanup function
    return () => {
      clearInterval(interval);
      unlistenSuccess.then((fn) => fn());
      unlistenError.then((fn) => fn());
    };
  }, []);

  const handleLogin = async () => {
    setIsLoading(true);
    setStatus("Opening browser...");

    try {
      const result = await invoke("login_with_google");
      setStatus("Browser opened, waiting for authentication...");
    } catch (error) {
      console.error("Error calling login_with_google:", error);
      setStatus("Error: " + String(error));
      setIsLoading(false);
    }
  };

  const handleOnboardingComplete = () => {
    setShowOnboarding(false);
    setIsOnboardingComplete(true);
    window.location.hash = "#/dashboard";
  };

  const handleSkipOnboarding = () => {
    setShowOnboarding(false);
    window.location.hash = "#/dashboard";
  };

  // Show onboarding if user is authenticated but hasn't completed onboarding
  if (showOnboarding) {
    return (
      <OnboardingWizard
        onComplete={handleOnboardingComplete}
        onSkip={handleSkipOnboarding}
      />
    );
  }

  return (
    <div
      className="min-h-screen bg-gradient-to-br from-zinc-900 via-zinc-800 to-zinc-900 flex items-center justify-center p-4 relative overflow-hidden rounded-md"
      style={{
        fontFamily: "BBH Sans Bartle",
        fontWeight: 400,
        fontStyle: "normal",
      }}
    >
      <div className="absolute top-0 right-0 w-[600px] h-[600px] bg-white/10 rounded-full blur-3xl translate-x-1/2 -translate-y-1/2" />
      <div className="absolute bottom-0 left-0 w-[600px] h-[600px] bg-white/10 rounded-full blur-3xl -translate-x-1/2 translate-y-1/2" />

      <div className="w-full max-w-md relative z-10">
        <div className="mb-6 max-w-md h-[400px] w-full flex flex-col items-center justify-center">
          <p className="text-4xl bg-[#1f1f20] font-bold rounded-xl py-2 px-4 mb-4">
            P
          </p>
          <h1 className="text-5xl font-bold text-center mb-4">
            Welcome to Pausa
          </h1>
          <p className="text-sm text-center opacity-50 mb-8">
            Focus. Breathe. Begin your next session.
          </p>
          <a
            onClick={handleLogin}
            className="w-fit text-sm text-center flex items-center justify-center gap-[2px] cursor-pointer opacity-50 hover:opacity-100 transition-opacity duration-700"
            style={{
              fontFamily: "Poppins",
              fontWeight: 500,
              fontStyle: "normal",
            }}
          >
            <span>Continue with Google</span>
            <ArrowRightIcon className="w-4 h-4 mt-[1px]" strokeWidth={3} />
          </a>
        </div>
      </div>
    </div>
  );
}
