import React, { useState, useRef, useEffect, useCallback } from "react";
import {
  Play,
  Pause,
  RotateCcw,
  MoreHorizontal,
  Shield,
  Target,
} from "lucide-react";
import { useAppStore, useCurrentSession, useCycleState } from "../store";
import { COLORS, FOCUS_WIDGET, SHADOWS, ANIMATIONS } from "../constants/design";
import { setupEventListeners } from "../lib/tauri";
import { SessionManager } from "../lib/commands";
import { CycleManager } from "../lib/cycleCommands";
import type { FocusSession, Position, CycleState } from "../types";
import { toastManager } from "../lib/toastManager";

interface FocusWidgetProps {
  session: FocusSession | null;
  onToggleSession: () => void;
  onResetSession: () => void;
  onOpenMenu: () => void;
}

interface CircularProgressProps {
  progress: number; // 0-100
  size: number;
  strokeWidth: number;
  isPreAlert?: boolean;
  isStrict?: boolean;
}

const CircularProgress: React.FC<CircularProgressProps> = ({
  progress,
  size,
  strokeWidth,
  isPreAlert = false,
  isStrict = false,
}) => {
  const radius = (size - strokeWidth) / 2;
  const circumference = radius * 2 * Math.PI;
  const strokeDasharray = circumference;
  const strokeDashoffset = circumference - (progress / 100) * circumference;

  return (
    <div className="relative" style={{ width: size, height: size }}>
      <svg className="transform -rotate-90" width={size} height={size}>
        {/* Background circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke={COLORS.gray[700]}
          strokeWidth={strokeWidth}
          fill="transparent"
        />

        {/* Progress circle */}
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          stroke={isPreAlert ? COLORS.warning : COLORS.primary[500]}
          strokeWidth={strokeWidth}
          fill="transparent"
          strokeDasharray={strokeDasharray}
          strokeDashoffset={strokeDashoffset}
          strokeLinecap="round"
          className={`transition-all duration-300 ${
            isPreAlert ? "animate-pulse" : ""
          }`}
        />

        {/* Pre-alert ring animation */}
        {isPreAlert && (
          <>
            <circle
              cx={size / 2}
              cy={size / 2}
              r={radius + 3}
              stroke={COLORS.warning}
              strokeWidth={2}
              fill="transparent"
              className="animate-ping opacity-60"
            />
            <circle
              cx={size / 2}
              cy={size / 2}
              r={radius + 6}
              stroke={COLORS.warning}
              strokeWidth={1}
              fill="transparent"
              className="animate-ping opacity-30"
              style={{ animationDelay: "0.5s" }}
            />
          </>
        )}
      </svg>

      {/* Strict mode indicator */}
      {isStrict && (
        <div className="absolute inset-0 flex items-center justify-center">
          <Shield className="w-3 h-3 text-red-500" />
        </div>
      )}
    </div>
  );
};

const formatTime = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes.toString().padStart(2, "0")}:${remainingSeconds
    .toString()
    .padStart(2, "0")}`;
};

// Widget positioning utilities
const STORAGE_KEY = "pausa-widget-position";
const SNAP_THRESHOLD = 20; // pixels
const EDGE_MARGIN = 10; // pixels from screen edge

const getStoredPosition = (): Position => {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const position = JSON.parse(stored) as Position;
      // Validate position is within current screen bounds
      if (
        position.x >= 0 &&
        position.x <= window.innerWidth - FOCUS_WIDGET.defaultSize.width &&
        position.y >= 0 &&
        position.y <= window.innerHeight - FOCUS_WIDGET.defaultSize.height
      ) {
        return position;
      }
    }
  } catch (error) {
    console.warn("Failed to load widget position from storage:", error);
  }

  // Default to center-right of screen
  return {
    x: window.innerWidth - FOCUS_WIDGET.defaultSize.width - 50,
    y: 100,
  };
};

const savePosition = (position: Position) => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(position));
  } catch (error) {
    console.warn("Failed to save widget position to storage:", error);
  }
};

const snapToEdges = (position: Position): Position => {
  const { x, y } = position;
  const maxX = window.innerWidth - FOCUS_WIDGET.defaultSize.width;
  const maxY = window.innerHeight - FOCUS_WIDGET.defaultSize.height;

  let snappedX = x;
  let snappedY = y;

  // Snap to left edge
  if (x < SNAP_THRESHOLD) {
    snappedX = EDGE_MARGIN;
  }
  // Snap to right edge
  else if (x > maxX - SNAP_THRESHOLD) {
    snappedX = maxX - EDGE_MARGIN;
  }

  // Snap to top edge
  if (y < SNAP_THRESHOLD) {
    snappedY = EDGE_MARGIN;
  }
  // Snap to bottom edge
  else if (y > maxY - SNAP_THRESHOLD) {
    snappedY = maxY - EDGE_MARGIN;
  }

  return { x: snappedX, y: snappedY };
};

export const FocusWidget: React.FC<FocusWidgetProps> = ({
  session,
  onToggleSession,
  onResetSession,
  onOpenMenu,
}) => {
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  const [position, setPosition] = useState<Position>(getStoredPosition);
  const [isHovered, setIsHovered] = useState(false);
  const widgetRef = useRef<HTMLDivElement>(null);
  const dragStartRef = useRef<Position | null>(null);

  // Calculate progress percentage
  const progress = session
    ? ((session.duration - session.remaining) / session.duration) * 100
    : 0;

  // Check if we're in pre-alert state
  const isPreAlert = session?.state === "pre-alert";

  // Check if session is in strict mode
  const isStrict = session?.isStrict || false;

  // Update position with bounds checking and snapping
  const updatePosition = useCallback((newPosition: Position) => {
    const maxX = window.innerWidth - FOCUS_WIDGET.defaultSize.width;
    const maxY = window.innerHeight - FOCUS_WIDGET.defaultSize.height;

    const boundedPosition = {
      x: Math.max(0, Math.min(newPosition.x, maxX)),
      y: Math.max(0, Math.min(newPosition.y, maxY)),
    };

    setPosition(boundedPosition);
    return boundedPosition;
  }, []);

  // Handle mouse down for dragging
  const handleMouseDown = (event: React.MouseEvent) => {
    if (!widgetRef.current) return;

    const rect = widgetRef.current.getBoundingClientRect();
    const offset = {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
    };

    setDragOffset(offset);
    setIsDragging(true);
    dragStartRef.current = position;

    // Prevent text selection during drag
    event.preventDefault();
  };

  // Handle dragging and snapping
  useEffect(() => {
    const handleMouseMove = (event: MouseEvent) => {
      if (!isDragging) return;

      const newPosition = {
        x: event.clientX - dragOffset.x,
        y: event.clientY - dragOffset.y,
      };

      updatePosition(newPosition);
    };

    const handleMouseUp = () => {
      if (isDragging) {
        // Apply edge snapping on mouse up
        const snappedPosition = snapToEdges(position);
        const finalPosition = updatePosition(snappedPosition);

        // Save position to localStorage
        savePosition(finalPosition);

        setIsDragging(false);
        dragStartRef.current = null;
      }
    };

    if (isDragging) {
      document.addEventListener("mousemove", handleMouseMove);
      document.addEventListener("mouseup", handleMouseUp);

      // Also handle mouse leave to ensure we don't get stuck in drag state
      document.addEventListener("mouseleave", handleMouseUp);
    }

    return () => {
      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
      document.removeEventListener("mouseleave", handleMouseUp);
    };
  }, [isDragging, dragOffset, position, updatePosition]);

  // Handle window resize to keep widget in bounds
  useEffect(() => {
    const handleResize = () => {
      const maxX = window.innerWidth - FOCUS_WIDGET.defaultSize.width;
      const maxY = window.innerHeight - FOCUS_WIDGET.defaultSize.height;

      if (position.x > maxX || position.y > maxY) {
        const newPosition = {
          x: Math.min(position.x, maxX),
          y: Math.min(position.y, maxY),
        };
        updatePosition(newPosition);
        savePosition(newPosition);
      }
    };

    window.addEventListener("resize", handleResize);
    return () => window.removeEventListener("resize", handleResize);
  }, [position, updatePosition]);

  // Handle visibility change to restore position
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        // Restore position when app becomes visible
        const storedPosition = getStoredPosition();
        updatePosition(storedPosition);
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () =>
      document.removeEventListener("visibilitychange", handleVisibilityChange);
  }, [updatePosition]);

  // Get cycle state from store
  const cycleState = useCycleState();

  // Don't render if no session
  if (!session) {
    return null;
  }

  return (
    <div
      ref={widgetRef}
      className={`
        fixed bg-gray-900 border border-gray-700 rounded-full
        flex items-center px-4 py-2 space-x-3
        transition-all duration-200 select-none
        ${
          isDragging
            ? "cursor-grabbing scale-105 shadow-2xl z-[9999]"
            : "cursor-grab hover:bg-gray-800 z-50"
        }
        ${isPreAlert ? "ring-2 ring-blue-400 ring-opacity-50" : ""}
        ${isHovered ? "shadow-lg" : ""}
      `}
      style={{
        left: position.x,
        top: position.y,
        width: FOCUS_WIDGET.defaultSize.width,
        height: FOCUS_WIDGET.defaultSize.height,
        boxShadow: isDragging ? SHADOWS.xl : SHADOWS.widget,
        animation: isPreAlert
          ? `pulse ${ANIMATIONS.duration.slow} ${ANIMATIONS.easing.easeInOut} infinite`
          : undefined,
        userSelect: "none",
        WebkitUserSelect: "none",
      }}
      onMouseDown={handleMouseDown}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
    >
      {/* Progress indicator */}
      <CircularProgress
        progress={progress}
        size={FOCUS_WIDGET.progressRingSize}
        strokeWidth={FOCUS_WIDGET.progressRingStroke}
        isPreAlert={isPreAlert}
        isStrict={isStrict}
      />

      {/* Time display */}
      <div className="flex-1 text-center">
        <div
          className={`
          font-mono text-lg font-semibold transition-colors duration-300
          ${isPreAlert ? "text-yellow-400 animate-pulse" : "text-white"}
        `}
        >
          {formatTime(session.remaining)}
        </div>
      </div>

      {/* Control buttons */}
      <div className="flex items-center space-x-1">
        {/* Play/Pause button */}
        <button
          onClick={(e) => {
            e.stopPropagation();
            onToggleSession();
          }}
          className={`
            p-1.5 rounded-full transition-colors duration-150
            hover:bg-gray-700 active:bg-gray-600
            ${session.isRunning ? "text-yellow-400" : "text-green-400"}
          `}
          title={session.isRunning ? "Pause session" : "Resume session"}
        >
          {session.isRunning ? (
            <Pause className="w-4 h-4" />
          ) : (
            <Play className="w-4 h-4" />
          )}
        </button>

        {/* Reset button */}
        <button
          onClick={(e) => {
            e.stopPropagation();
            onResetSession();
          }}
          className="
            p-1.5 rounded-full transition-colors duration-150
            hover:bg-gray-700 active:bg-gray-600 text-gray-400
            hover:text-red-400
          "
          title="Reset session"
        >
          <RotateCcw className="w-4 h-4" />
        </button>

        {/* Menu button */}
        <button
          onClick={(e) => {
            e.stopPropagation();
            onOpenMenu();
          }}
          className="
            p-1.5 rounded-full transition-colors duration-150
            hover:bg-gray-700 active:bg-gray-600 text-gray-400
            hover:text-white
          "
          title="More options"
        >
          <MoreHorizontal className="w-4 h-4" />
        </button>
      </div>

      {/* Strict mode indicator badge */}
      {isStrict && (
        <div className="absolute -top-1 -right-1 w-3 h-3 bg-red-500 rounded-full border-2 border-gray-900" />
      )}

      {/* Cycle counter badge */}
      {cycleState && cycleState.cycle_count > 0 && (
        <div className="absolute -bottom-2 left-1/2 transform -translate-x-1/2 flex items-center space-x-1 bg-gray-800 border border-gray-600 rounded-full px-2 py-0.5">
          <Target className="w-3 h-3 text-blue-400" />
          <span className="text-xs font-semibold text-blue-400">
            {cycleState.cycle_count}
          </span>
        </div>
      )}
    </div>
  );
};

// Hook to use the FocusWidget with store integration and real-time updates
export const useFocusWidget = () => {
  const session = useCurrentSession();
  const { setCurrentSession, setCurrentBreak, showFocusWidget, hideFocusWidget } = useAppStore();

  // Set up real-time session updates from backend
  useEffect(() => {
    let unsubscribe: (() => void) | null = null;

    const setupEventListener = async () => {
      try {
        unsubscribe = await setupEventListeners((event) => {
          switch (event.type) {
            case "session-update":
              setCurrentSession(event.session);
              // Show widget when session starts
              if (event.session && event.session.isRunning) {
                showFocusWidget();
              }
              break;
            case "break-update":
              setCurrentBreak(event.breakSession);
              // Hide widget during breaks
              hideFocusWidget();
              break;
            case "state-change":
              console.log(`State changed from ${event.from} to ${event.to}`);
              // Handle specific state transitions
              if (event.to === "idle") {
                hideFocusWidget();
              }
              break;
          }
        });
      } catch (error) {
        console.error("Failed to setup event listeners:", error);
        toastManager.showError(
          "Real-time updates are temporarily unavailable. Session status may be outdated.",
          { title: "Live Updates Disabled", duration: 6000 }
        );
      }
    };

    setupEventListener();

    return () => {
      if (unsubscribe) {
        unsubscribe();
      }
    };
  }, [setCurrentSession, setCurrentBreak, showFocusWidget, hideFocusWidget]);

  // Sync session state on mount
  useEffect(() => {
    const syncSessionState = async () => {
      try {
        const currentSession = await SessionManager.getCurrentSession();
        const currentBreak = await SessionManager.getCurrentBreak();

        setCurrentSession(currentSession);
        setCurrentBreak(currentBreak);

        // Show widget if there's an active session
        if (currentSession && currentSession.isRunning) {
          showFocusWidget();
        } else {
          hideFocusWidget();
        }
      } catch (error) {
        console.error("Failed to sync session state:", error);
        // In case of error, hide the widget to avoid showing stale data
        hideFocusWidget();
      }
    };

    syncSessionState();
  }, [setCurrentSession, setCurrentBreak, showFocusWidget, hideFocusWidget]);

  const handleToggleSession = async () => {
    try {
      await SessionManager.toggleSession();
      // State will be updated via event listener
    } catch (error) {
      console.error("Failed to toggle session:", error);
      toastManager.showError(
        "We couldn't toggle the focus session. Please try again.",
        { title: "Toggle Focus Failed" }
      );
    }
  };

  const handleResetSession = async () => {
    try {
      await SessionManager.resetSession();
      // State will be updated via event listener
    } catch (error) {
      console.error("Failed to reset session:", error);
      toastManager.showError(
        "We couldn't reset the current session. Please try again.",
        { title: "Reset Session Failed" }
      );
    }
  };

  const handleOpenMenu = () => {
    window.location.hash = '#/settings';
  };

  return {
    session,
    onToggleSession: handleToggleSession,
    onResetSession: handleResetSession,
    onOpenMenu: handleOpenMenu,
    showWidget: showFocusWidget,
    hideWidget: hideFocusWidget,
  };
};
