export function ShortcutsCard() {
  return (
    <section className="bg-gray-900/40 border border-gray-800 rounded-xl p-5">
      <h2 className="text-sm font-semibold text-gray-300 mb-3">Shortcuts</h2>
      <ul className="text-sm text-gray-400 space-y-2">
        <li>
          <span className="text-gray-500">⌘</span> +{" "}
          <span className="text-gray-500">Space</span> — Command Palette
        </li>
        <li>
          <span className="text-gray-500">⌘</span> +{" "}
          <span className="text-gray-500">⇧F</span> — Toggle Focus
        </li>
        <li>
          <span className="text-gray-500">⌘</span> +{" "}
          <span className="text-gray-500">⇧L</span> — Lock Now
        </li>
        <li>
          <span className="text-gray-500">⌘</span> +{" "}
          <span className="text-gray-500">P</span> — Open Settings
        </li>
      </ul>
    </section>
  );
}
