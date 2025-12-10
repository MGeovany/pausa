import React, { useState, useEffect, useRef, useMemo } from "react";
import { Search, Clock, Lock, Coffee, BarChart3, Settings } from "lucide-react";
import { ANIMATIONS, COMMAND_PALETTE, SHADOWS } from "../constants/design";
import type { Command } from "../types";
import { notificationHelper } from "../lib/notificationHelper";

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  commands: Command[];
}

interface CommandItemProps {
  command: Command;
  isSelected: boolean;
  onClick: () => void;
}

interface CommandGroupProps {
  category: string;
  commands: Command[];
  selectedIndex: number;
  onCommandClick: (command: Command) => void;
  startIndex: number;
}

const CommandItem: React.FC<CommandItemProps> = ({
  command,
  isSelected,
  onClick,
}) => {
  const getIcon = (category: Command["category"]) => {
    switch (category) {
      case "focus":
        return <Clock className="w-4 h-4" />;
      case "lock":
        return <Lock className="w-4 h-4" />;
      case "break":
        return <Coffee className="w-4 h-4" />;
      case "stats":
        return <BarChart3 className="w-4 h-4" />;
      case "settings":
        return <Settings className="w-4 h-4" />;
      default:
        return <Clock className="w-4 h-4" />;
    }
  };

  const getCategoryColor = (category: Command["category"]) => {
    switch (category) {
      case "focus":
        return isSelected ? "text-white" : "text-blue-400";
      case "lock":
        return isSelected ? "text-white" : "text-red-400";
      case "break":
        return isSelected ? "text-white" : "text-green-400";
      case "stats":
        return isSelected ? "text-white" : "text-purple-400";
      case "settings":
        return isSelected ? "text-white" : "text-gray-400";
      default:
        return isSelected ? "text-white" : "text-gray-400";
    }
  };

  return (
    <div
      className={`
        flex items-center justify-between px-4 py-3 cursor-pointer transition-all duration-150
        ${
          isSelected
            ? "bg-blue-500 text-white transform scale-[1.02]"
            : "text-gray-300 hover:bg-gray-800/50"
        }
        rounded-lg mx-2 my-1
      `}
      onClick={onClick}
      style={{
        height: COMMAND_PALETTE.itemHeight - 8,
        animation: isSelected
          ? `pulse ${ANIMATIONS.duration.normal} ${ANIMATIONS.easing.easeOut}`
          : undefined,
      }}
    >
      <div className="flex items-center space-x-3">
        <div className={getCategoryColor(command.category)}>
          {getIcon(command.category)}
        </div>
        <span className="font-medium">{command.label}</span>
      </div>
      {command.shortcut && (
        <kbd
          className={`
          px-2 py-1 text-xs font-mono rounded border transition-all duration-150
          ${
            isSelected
              ? "bg-blue-600/50 border-blue-400/50 text-blue-100"
              : "bg-gray-700/50 border-gray-600/50 text-gray-400"
          }
        `}
        >
          {command.shortcut}
        </kbd>
      )}
    </div>
  );
};

const CommandGroup: React.FC<CommandGroupProps> = ({
  category,
  commands,
  selectedIndex,
  onCommandClick,
  startIndex,
}) => {
  const getCategoryLabel = (category: string) => {
    switch (category) {
      case "focus":
        return "Focus";
      case "break":
        return "Break";
      case "lock":
        return "Lock";
      case "stats":
        return "Statistics";
      case "settings":
        return "Settings";
      default:
        return category.charAt(0).toUpperCase() + category.slice(1);
    }
  };

  return (
    <div className="mb-2">
      <div className="px-4 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider">
        {getCategoryLabel(category)}
      </div>
      {commands.map((command, index) => {
        const globalIndex = startIndex + index;
        return (
          <CommandItem
            key={command.id}
            command={command}
            isSelected={selectedIndex === globalIndex}
            onClick={() => onCommandClick(command)}
          />
        );
      })}
    </div>
  );
};

export const CommandPalette: React.FC<CommandPaletteProps> = ({
  isOpen,
  onClose,
  commands,
}) => {
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);

  // Filter commands based on search query with fuzzy matching
  const filteredCommands = useMemo(() => {
    if (!searchQuery.trim()) {
      return commands;
    }

    const query = searchQuery.toLowerCase();
    return commands.filter((command) => {
      const label = command.label.toLowerCase();
      const category = command.category.toLowerCase();

      // Exact match
      if (label.includes(query) || category.includes(query)) {
        return true;
      }

      // Fuzzy match - check if query characters appear in order
      return fuzzyMatch(query, label) || fuzzyMatch(query, category);
    });
  }, [searchQuery, commands]);

  // Group commands by category
  const groupedCommands = useMemo(() => {
    const groups: Record<string, Command[]> = {};

    filteredCommands.forEach((command) => {
      if (!groups[command.category]) {
        groups[command.category] = [];
      }
      groups[command.category].push(command);
    });

    // Sort categories in a logical order
    const categoryOrder = ["focus", "break", "lock", "stats", "settings"];
    const sortedGroups: Record<string, Command[]> = {};

    categoryOrder.forEach((category) => {
      if (groups[category]) {
        sortedGroups[category] = groups[category];
      }
    });

    // Add any remaining categories
    Object.keys(groups).forEach((category) => {
      if (!sortedGroups[category]) {
        sortedGroups[category] = groups[category];
      }
    });

    return sortedGroups;
  }, [filteredCommands]);

  const totalCommands = filteredCommands.length;

  // Simple fuzzy matching function
  const fuzzyMatch = (query: string, target: string): boolean => {
    let queryIndex = 0;
    let targetIndex = 0;

    while (queryIndex < query.length && targetIndex < target.length) {
      if (query[queryIndex] === target[targetIndex]) {
        queryIndex++;
      }
      targetIndex++;
    }

    return queryIndex === query.length;
  };

  // Reset selection when filtered commands change
  useEffect(() => {
    setSelectedIndex(0);
  }, [totalCommands]);

  // Focus search input when palette opens
  useEffect(() => {
    if (isOpen && searchInputRef.current) {
      searchInputRef.current.focus();
    }
  }, [isOpen]);

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!isOpen) return;

      switch (event.key) {
        case "ArrowDown":
          event.preventDefault();
          setSelectedIndex((prev) => (prev < totalCommands - 1 ? prev + 1 : 0));
          break;
        case "ArrowUp":
          event.preventDefault();
          setSelectedIndex((prev) => (prev > 0 ? prev - 1 : totalCommands - 1));
          break;
        case "Enter":
          event.preventDefault();
          if (filteredCommands[selectedIndex]) {
            handleCommandExecute(filteredCommands[selectedIndex]);
          }
          break;
        case "Escape":
          event.preventDefault();
          handleClose();
          break;
      }
    };

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isOpen, totalCommands, selectedIndex, filteredCommands]);

  const handleCommandExecute = async (command: Command) => {
    try {
      await command.action();
      handleClose();
    } catch (error) {
      console.error("Failed to execute command:", error);
      notificationHelper.showError(
        "Command Execution Failed",
        "The command couldn't be completed. Please try again."
      );
    }
  };

  const handleClose = () => {
    onClose();
    setSearchQuery("");
    setSelectedIndex(0);
  };

  const handleSearchChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setSearchQuery(event.target.value);
  };

  const handleBackdropClick = (event: React.MouseEvent) => {
    if (event.target === event.currentTarget) {
      handleClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-32 bg-black/60 backdrop-blur-sm"
      onClick={handleBackdropClick}
      style={{
        animation: `fadeIn ${ANIMATIONS.duration.normal} ${ANIMATIONS.easing.easeOut}`,
      }}
    >
      <div
        ref={containerRef}
        className="bg-gray-950/95 backdrop-blur-xl rounded-2xl border border-gray-800/50 overflow-hidden"
        style={{
          width: COMMAND_PALETTE.maxWidth,
          maxHeight: COMMAND_PALETTE.maxHeight,
          boxShadow: SHADOWS.raycast,
          animation: `slideIn ${ANIMATIONS.duration.normal} ${ANIMATIONS.easing.easeOut}`,
        }}
      >
        {/* Search Input */}
        <div className="flex items-center px-6 py-4 border-b border-gray-800/50">
          <Search className="w-5 h-5 text-gray-400 mr-3 flex-shrink-0" />
          <input
            ref={searchInputRef}
            type="text"
            placeholder="Search commands..."
            value={searchQuery}
            onChange={handleSearchChange}
            className="flex-1 bg-transparent text-white placeholder-gray-500 outline-none text-lg font-medium"
            style={{
              fontFamily: "Inter, SF Pro Display, system-ui, sans-serif",
            }}
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery("")}
              className="ml-2 text-gray-500 hover:text-gray-300 transition-colors"
            >
              ✕
            </button>
          )}
        </div>

        {/* Command List */}
        <div
          className="max-h-96 overflow-y-auto py-2"
          style={{ scrollbarWidth: "thin" }}
        >
          {totalCommands > 0 ? (
            Object.entries(groupedCommands).map(
              ([category, commands], groupIndex) => {
                const startIndex = Object.entries(groupedCommands)
                  .slice(0, groupIndex)
                  .reduce((acc, [, cmds]) => acc + cmds.length, 0);

                return (
                  <CommandGroup
                    key={category}
                    category={category}
                    commands={commands}
                    selectedIndex={selectedIndex}
                    onCommandClick={handleCommandExecute}
                    startIndex={startIndex}
                  />
                );
              }
            )
          ) : (
            <div className="px-6 py-12 text-center text-gray-500">
              <Search className="w-12 h-12 mx-auto mb-4 opacity-30" />
              <p className="text-lg font-medium mb-1">No commands found</p>
              <p className="text-sm">Try a different search term</p>
            </div>
          )}
        </div>

        {/* Footer with hints */}
        {totalCommands > 0 && (
          <div className="px-6 py-3 border-t border-gray-800/50 bg-gray-900/50">
            <div className="flex items-center justify-between text-xs text-gray-500">
              <div className="flex items-center space-x-4">
                <span className="flex items-center space-x-1">
                  <kbd className="px-1.5 py-0.5 bg-gray-800 rounded text-xs">
                    ↑↓
                  </kbd>
                  <span>navigate</span>
                </span>
                <span className="flex items-center space-x-1">
                  <kbd className="px-1.5 py-0.5 bg-gray-800 rounded text-xs">
                    ↵
                  </kbd>
                  <span>select</span>
                </span>
              </div>
              <span className="flex items-center space-x-1">
                <kbd className="px-1.5 py-0.5 bg-gray-800 rounded text-xs">
                  esc
                </kbd>
                <span>close</span>
              </span>
            </div>
          </div>
        )}
      </div>

      <style>{`
        @keyframes fadeIn {
          from { opacity: 0; }
          to { opacity: 1; }
        }
        
        @keyframes slideIn {
          from { 
            opacity: 0; 
            transform: translateY(-20px) scale(0.96); 
          }
          to { 
            opacity: 1; 
            transform: translateY(0) scale(1); 
          }
        }

        @keyframes pulse {
          0%, 100% { transform: scale(1.02); }
          50% { transform: scale(1.04); }
        }

        /* Custom scrollbar */
        .max-h-96::-webkit-scrollbar {
          width: 6px;
        }
        
        .max-h-96::-webkit-scrollbar-track {
          background: transparent;
        }
        
        .max-h-96::-webkit-scrollbar-thumb {
          background: rgba(107, 114, 128, 0.3);
          border-radius: 3px;
        }
        
        .max-h-96::-webkit-scrollbar-thumb:hover {
          background: rgba(107, 114, 128, 0.5);
        }
      `}</style>
    </div>
  );
};
