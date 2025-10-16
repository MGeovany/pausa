
import { useEffect } from 'react';
import { TauriCommandTest } from './components/TauriCommandTest';
import { FocusWidget, useFocusWidget } from './components/FocusWidget';
import { CommandPalette } from './components/CommandPalette';
import { useAppStore, useUIState } from './store';
import { useCommands } from './lib/commands';

function App() {
  const { session, onToggleSession, onResetSession, onOpenMenu } = useFocusWidget();
  const { 
    isCommandPaletteOpen, 
    isFocusWidgetVisible 
  } = useUIState();
  const { toggleCommandPalette } = useAppStore();
  const { getAllCommands } = useCommands();

  // Global hotkey handling
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      // Command/Ctrl + Space for command palette
      if ((event.metaKey || event.ctrlKey) && event.code === 'Space') {
        event.preventDefault();
        toggleCommandPalette();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [toggleCommandPalette]);

  return (
    <main className="bg-white min-h-screen">
      <TauriCommandTest />
      
      {/* Focus Widget - only show when visible and session exists */}
      {isFocusWidgetVisible && session && (
        <FocusWidget
          session={session}
          onToggleSession={onToggleSession}
          onResetSession={onResetSession}
          onOpenMenu={onOpenMenu}
        />
      )}

      {/* Command Palette */}
      <CommandPalette
        isOpen={isCommandPaletteOpen}
        onClose={() => toggleCommandPalette()}
        commands={getAllCommands()}
      />
    </main>
  );
}

export default App;
