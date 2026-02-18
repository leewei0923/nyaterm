import { useEffect, useRef, useState } from "react";
import { useTheme } from "../../context/ThemeContext";

interface HeaderProps {
  onNewSession: () => void;
}

interface MenuItem {
  label: string;
  action?: () => void;
  separator?: boolean;
}

/** Top bar with File/Edit/View/Terminal/Help menus, theme picker, and mobile toggles. */
export default function Header({
  onNewSession,
  onToggleLeft,
  onToggleRight,
}: HeaderProps & { onToggleLeft?: () => void; onToggleRight?: () => void }) {
  const [activeMenu, setActiveMenu] = useState<string | null>(null);
  const [showThemePicker, setShowThemePicker] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);
  const themeRef = useRef<HTMLDivElement>(null);
  const { themeName, setTheme, themeNames } = useTheme();

  const menus: Record<string, MenuItem[]> = {
    File: [
      { label: "New SSH Connection", action: onNewSession },
      { label: "separator", separator: true },
      { label: "Exit" },
    ],
    Edit: [{ label: "Copy" }, { label: "Paste" }, { label: "Select All" }],
    View: [{ label: "Toggle Sidebar" }, { label: "Toggle Panel" }],
    Terminal: [
      { label: "New SSH Connection", action: onNewSession },
      { label: "New Local Terminal", action: onNewSession },
    ],
    Help: [{ label: "About" }],
  };

  // Close menu when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setActiveMenu(null);
      }
      if (themeRef.current && !themeRef.current.contains(e.target as Node)) {
        setShowThemePicker(false);
      }
    };
    if (activeMenu || showThemePicker) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [activeMenu, showThemePicker]);

  return (
    <header
      className="h-10 border-b flex items-center justify-between px-3 select-none shrink-0"
      style={{ backgroundColor: "var(--df-bg-panel)", borderColor: "var(--df-border)" }}
    >
      <div className="flex items-center gap-4" ref={menuRef}>
        {/* Mobile Left Toggle */}
        <button
          className="lg:hidden"
          style={{ color: "var(--df-text-muted)" }}
          onClick={onToggleLeft}
        >
          <span className="material-icons text-xl">menu</span>
        </button>

        <nav className="flex items-center gap-0 text-xs font-medium relative">
          {Object.keys(menus).map((item) => (
            <div key={item} className="relative">
              <span
                className="cursor-pointer px-2 py-1 rounded transition-colors"
                style={{
                  color: activeMenu === item ? "var(--df-primary)" : "var(--df-text-muted)",
                  backgroundColor:
                    activeMenu === item
                      ? "color-mix(in srgb, var(--df-primary) 10%, transparent)"
                      : undefined,
                }}
                onClick={() => setActiveMenu(activeMenu === item ? null : item)}
              >
                {item}
              </span>
              {activeMenu === item && (
                <div
                  className="absolute top-full left-0 mt-1 rounded shadow-xl py-1 min-w-[180px] z-50 border"
                  style={{ backgroundColor: "var(--df-bg-panel)", borderColor: "var(--df-border)" }}
                >
                  {menus[item].map((menuItem) =>
                    menuItem.separator ? (
                      <div
                        key={`sep-${menuItem.label}`}
                        className="my-1 border-t"
                        style={{ borderColor: "var(--df-border)" }}
                      />
                    ) : (
                      <div
                        key={menuItem.label}
                        className="px-3 py-1.5 text-xs cursor-pointer transition-colors"
                        style={{ color: "var(--df-text)" }}
                        onMouseEnter={(e) => {
                          e.currentTarget.style.backgroundColor =
                            "color-mix(in srgb, var(--df-primary) 20%, transparent)";
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.backgroundColor = "";
                        }}
                        onClick={() => {
                          menuItem.action?.();
                          setActiveMenu(null);
                        }}
                      >
                        {menuItem.label}
                      </div>
                    ),
                  )}
                </div>
              )}
            </div>
          ))}
        </nav>
      </div>
      <div className="flex items-center gap-3" style={{ color: "var(--df-text-muted)" }}>
        {/* Mobile Right Toggle */}
        <button
          className="md:hidden"
          style={{ color: "var(--df-text-muted)" }}
          onClick={onToggleRight}
        >
          <span className="material-icons text-xl">view_sidebar</span>
        </button>

        <span className="material-icons text-base cursor-pointer hover:opacity-80 transition-opacity hidden sm:block">
          search
        </span>

        {/* Theme Picker */}
        <div className="relative inline-flex items-center" ref={themeRef}>
          <span
            className="material-icons text-base cursor-pointer hover:opacity-80 transition-opacity hidden sm:block"
            onClick={() => setShowThemePicker(!showThemePicker)}
            title="Switch Theme"
          >
            palette
          </span>
          {showThemePicker && (
            <div
              className="absolute top-full right-0 mt-2 rounded-lg shadow-2xl py-1.5 min-w-[200px] z-50 border"
              style={{ backgroundColor: "var(--df-bg-panel)", borderColor: "var(--df-border)" }}
            >
              <div
                className="px-3 py-1.5 text-[10px] uppercase tracking-wider font-bold"
                style={{ color: "var(--df-text-dimmed)" }}
              >
                Theme
              </div>
              {themeNames.map((t) => (
                <div
                  key={t.id}
                  className="flex items-center gap-2.5 px-3 py-2 cursor-pointer transition-colors"
                  style={{
                    backgroundColor:
                      themeName === t.id
                        ? "color-mix(in srgb, var(--df-primary) 12%, transparent)"
                        : undefined,
                  }}
                  onMouseEnter={(e) => {
                    if (themeName !== t.id)
                      e.currentTarget.style.backgroundColor = "var(--df-bg-hover)";
                  }}
                  onMouseLeave={(e) => {
                    if (themeName !== t.id) e.currentTarget.style.backgroundColor = "";
                  }}
                  onClick={() => {
                    setTheme(t.id);
                    setShowThemePicker(false);
                  }}
                >
                  <div
                    className="w-4 h-4 rounded-full border shrink-0"
                    style={{ backgroundColor: t.swatch, borderColor: "var(--df-border)" }}
                  />
                  <span
                    className="text-xs font-medium flex-1"
                    style={{ color: themeName === t.id ? "var(--df-primary)" : "var(--df-text)" }}
                  >
                    {t.name}
                  </span>
                  {themeName === t.id && (
                    <span className="material-icons text-sm" style={{ color: "var(--df-primary)" }}>
                      check
                    </span>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <span className="material-icons text-base cursor-pointer hover:opacity-80 transition-opacity hidden sm:block">
          settings
        </span>
        <span className="material-icons text-base cursor-pointer hover:opacity-80 transition-opacity hidden sm:block">
          fullscreen
        </span>
      </div>
    </header>
  );
}
