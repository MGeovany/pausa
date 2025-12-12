export function ShortcutsCard() {
  const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
  const cmdKey = isMac ? "⌘" : "Ctrl";
  const shiftKey = "⇧";

  return (
    <section className="bg-gray-900/40 border border-gray-800 rounded-xl p-5">
      <h2 className="text-sm font-semibold text-gray-300 mb-3">
        Keyboard Shortcuts
      </h2>
      <ul className="text-sm text-gray-400 space-y-3">
        <li>
          <kbd className="px-2 py-1 text-xs font-mono bg-gray-800 border border-gray-700 rounded">
            {cmdKey} K
          </kbd>{" "}
          — Command Palette
        </li>
        <li>
          <kbd className="px-2 py-1 text-xs font-mono bg-gray-800 border border-gray-700 rounded">
            {cmdKey} {shiftKey} F
          </kbd>{" "}
          — Start Focus Session
        </li>

        <li>
          <kbd className="px-2 py-1 text-xs font-mono bg-gray-800 border border-gray-700 rounded">
            {cmdKey} ,
          </kbd>{" "}
          — Settings
        </li>
        <li>
          <kbd className="px-2 py-1 text-xs font-mono bg-gray-800 border border-gray-700 rounded">
            {cmdKey} {shiftKey} S
          </kbd>{" "}
          — Statistics
        </li>
      </ul>
    </section>
  );
}
