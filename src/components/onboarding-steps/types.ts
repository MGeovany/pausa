export interface StepProps {
  onNext: () => void;
  onPrevious: () => void;
  canGoNext: boolean;
  canGoPrevious: boolean;
  stepData: any;
  setStepData: (data: any) => void;
}
