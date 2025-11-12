import React, { Component, ErrorInfo, ReactNode } from "react";
import { errorHandler } from "../lib/errorHandler";
import { AlertCircle, RefreshCw, X } from "lucide-react";

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

/**
 * Error Boundary component to catch and handle React errors
 */
export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
      errorInfo: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    return {
      hasError: true,
      error,
      errorInfo: null,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Log error to error handler
    errorHandler.logError(
      error,
      "ErrorBoundary",
      "React component error",
      "error"
    );

    this.setState({
      error,
      errorInfo,
    });
  }

  handleReset = () => {
    this.setState({
      hasError: false,
      error: null,
      errorInfo: null,
    });
  };

  handleReload = () => {
    window.location.reload();
  };

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      const strategy = errorHandler.getRecoveryStrategy(
        this.state.error || "Unknown error"
      );

      return (
        <div className="min-h-screen bg-gray-900 flex items-center justify-center p-4">
          <div className="bg-gray-800 border border-red-500/30 rounded-xl p-8 max-w-2xl w-full animate-scale-in">
            <div className="flex items-start space-x-4">
              <div className="flex-shrink-0">
                <AlertCircle className="w-8 h-8 text-red-400" />
              </div>
              <div className="flex-1">
                <h2 className="text-2xl font-semibold text-white mb-2">
                  Something went wrong
                </h2>
                <p className="text-gray-300 mb-4">{strategy.userMessage}</p>

                {/* Error details (collapsible) */}
                <details className="mb-6">
                  <summary className="cursor-pointer text-sm text-gray-400 hover:text-gray-300 mb-2">
                    Technical Details
                  </summary>
                  <div className="bg-gray-900 rounded-lg p-4 text-sm font-mono text-gray-400 overflow-auto max-h-48">
                    <p className="mb-2">
                      <strong>Error:</strong> {this.state.error?.message}
                    </p>
                    {this.state.error?.stack && (
                      <pre className="text-xs whitespace-pre-wrap">
                        {this.state.error.stack}
                      </pre>
                    )}
                  </div>
                </details>

                {/* Suggested actions */}
                {strategy.suggestedActions.length > 0 && (
                  <div className="mb-6">
                    <h3 className="text-sm font-semibold text-gray-300 mb-2">
                      What you can do:
                    </h3>
                    <ul className="list-disc list-inside space-y-1 text-sm text-gray-400">
                      {strategy.suggestedActions.map((action, index) => (
                        <li key={index}>{action}</li>
                      ))}
                    </ul>
                  </div>
                )}

                {/* Action buttons */}
                <div className="flex space-x-3">
                  {strategy.canRecover && (
                    <button
                      onClick={this.handleReset}
                      className="flex items-center space-x-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors"
                    >
                      <RefreshCw className="w-4 h-4" />
                      <span>Try Again</span>
                    </button>
                  )}
                  <button
                    onClick={this.handleReload}
                    className="flex items-center space-x-2 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white font-medium rounded-lg transition-colors"
                  >
                    <RefreshCw className="w-4 h-4" />
                    <span>Reload App</span>
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

/**
 * Toast notification for errors
 */
interface ErrorToastProps {
  error: string;
  onClose: () => void;
  recoveryActions?: string[];
}

export const ErrorToast: React.FC<ErrorToastProps> = ({
  error,
  onClose,
  recoveryActions,
}) => {
  const strategy = errorHandler.getRecoveryStrategy(error);

  return (
    <div className="fixed bottom-4 right-4 z-50 animate-slide-in-up">
      <div className="bg-red-900/90 backdrop-blur-sm border border-red-700 rounded-lg p-4 max-w-md shadow-xl">
        <div className="flex items-start space-x-3">
          <AlertCircle className="w-5 h-5 text-red-400 flex-shrink-0 mt-0.5" />
          <div className="flex-1">
            <h4 className="text-sm font-semibold text-red-200 mb-1">Error</h4>
            <p className="text-sm text-red-100 mb-2">{strategy.userMessage}</p>
            {recoveryActions && recoveryActions.length > 0 && (
              <ul className="text-xs text-red-200 space-y-1 mb-2">
                {recoveryActions.map((action, index) => (
                  <li key={index}>• {action}</li>
                ))}
              </ul>
            )}
          </div>
          <button
            onClick={onClose}
            className="text-red-400 hover:text-red-300 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>
  );
};
