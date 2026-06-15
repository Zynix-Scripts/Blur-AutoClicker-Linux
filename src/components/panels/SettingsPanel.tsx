import "./SettingsPanel.css";
import type { AppInfo, ClickerStatus, Settings } from "../../store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useMemo, useRef, useState } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import ConfirmDialog from "../ConfirmDialog";
import { changelogEntries } from "../../changelog";
import ChangelogContent from "../ChangelogContent";

interface CumulativeStats {
  totalClicks: number;
  totalTimeSecs: number;
  totalSessions: number;
  avgCpu: number;
}

interface Props {
  settings: Settings;
  update: (patch: Partial<Settings>) => void;
  app_info: AppInfo;
  onReset: () => Promise<void>;
  updateCheckStatus: "idle" | "checking" | "available" | "unavailable" | "error";
  onCheckForUpdate: () => void;
}

function format_time(total_seconds: number): string {
  if (total_seconds < 0.01) return "0s";
  if (total_seconds < 60) {
    return `${Math.floor(total_seconds)}s`;
  }
  if (total_seconds < 3600) {
    const m = Math.floor(total_seconds / 60);
    const s = Math.floor(total_seconds % 60);
    return s > 0 ? `${m}m ${s}s` : `${m}m`;
  }
  const h = Math.floor(total_seconds / 3600);
  const m = Math.floor((total_seconds % 3600) / 60);
  return m > 0 ? `${h}h ${m}m` : `${h}h`;
}

function format_number(n: number): string {
  return Math.floor(n).toLocaleString();
}

function format_cpu(cpu: number): string {
  if (cpu < 0) return "N/A";
  return `${cpu.toFixed(1)}%`;
}

function format_decimal(n: number, digits = 1): string {
  if (!Number.isFinite(n) || n === 0) return "0";
  return n.toFixed(digits);
}

const TABS = [
  { key: "general", label: "General" },
  { key: "appearance", label: "Appearance" },
  { key: "overlay", label: "Overlay" },
  { key: "stats", label: "Stats" },
  { key: "about", label: "About" },
] as const;

type TabKey = (typeof TABS)[number]["key"];

const THEMES = [
  { key: "dark", label: "Default" },
  { key: "oled", label: "OLED Black" },
  { key: "catppuccin-mocha", label: "Catppuccin Mocha" },
  { key: "light", label: "Light" },
  { key: "nord", label: "Nord" },
  { key: "gruvbox-dark", label: "Gruvbox Dark" },
  { key: "tokyo-night", label: "Tokyo Night" },
] as const;

export default function SettingsPanel({
  settings,
  update,
  app_info,
  onReset,
  updateCheckStatus,
  onCheckForUpdate,
}: Props) {
  const [resetting, set_resetting] = useState(false);
  const [resetting_stats, set_resetting_stats] = useState(false);
  const [stats, set_stats] = useState<CumulativeStats | null>(null);
  const [at_bottom, set_at_bottom] = useState(false);
  const [active_tab, set_active_tab] = useState<TabKey>("general");
  const [pending_action, set_pending_action] = useState<
    "reset-settings" | "clear-stats" | null
  >(null);
  const [show_changelog, set_show_changelog] = useState(false);

  const panel_ref = useRef<HTMLDivElement>(null);
  const prev_running = useRef(false);

  const refresh_stats = () => {
    invoke<CumulativeStats>("get_stats")
      .then(set_stats)
      .catch(() => {});
  };

  useEffect(() => {
    refresh_stats();
  }, []);

  useEffect(() => {
    if (active_tab === "stats") {
      refresh_stats();
    }
  }, [active_tab]);

  useEffect(() => {
    let cleanup: (() => void) | undefined;

    listen<ClickerStatus>("clicker-status", (event) => {
      const payload = event.payload;
      if (prev_running.current && !payload.running) {
        refresh_stats();
      }
      prev_running.current = payload.running;
    })
      .then((unlisten) => {
        cleanup = unlisten;
      })
      .catch(() => {});

    return () => {
      cleanup?.();
    };
  }, []);

  const handle_scroll = () => {
    const el = panel_ref.current;
    if (!el) return;
    set_at_bottom(el.scrollTop + el.clientHeight >= el.scrollHeight - 2);
  };

  const has_stats = stats !== null && stats.totalSessions > 0;

  const derived = useMemo(() => {
    if (!has_stats) return null;
    return {
      clicks_per_session:
        stats.totalSessions > 0 ? stats.totalClicks / stats.totalSessions : 0,
      time_per_session:
        stats.totalSessions > 0 ? stats.totalTimeSecs / stats.totalSessions : 0,
      cps: stats.totalTimeSecs > 0 ? stats.totalClicks / stats.totalTimeSecs : 0,
    };
  }, [stats, has_stats]);

  const handle_reset_stats = () => {
    set_pending_action("clear-stats");
  };

  const confirm_reset_stats = () => {
    set_pending_action(null);
    set_resetting_stats(true);
    invoke<CumulativeStats>("reset_stats")
      .then(set_stats)
      .finally(() => set_resetting_stats(false));
  };

  const confirm_reset_settings = () => {
    set_pending_action(null);
    set_resetting(true);
    onReset().finally(() => set_resetting(false));
  };

  const update_button_label = {
    idle: "Check for Updates",
    checking: "Checking...",
    available: "Update Available",
    unavailable: "No Update Available",
    error: "Check Failed",
  }[updateCheckStatus];

  return (
    <div className="settings-layout">
      <div className="settings-sidebar">
        {TABS.map((t) => (
          <button
            key={t.key}
            className={`settings-tab ${active_tab === t.key ? "active" : ""}`}
            onClick={() => set_active_tab(t.key)}
          >
            {t.label}
          </button>
        ))}
      </div>

      <div className="settings-wrapper">
        <div className="settings-panel" ref={panel_ref} onScroll={handle_scroll}>
          {active_tab === "general" && (
            <>
              <div className="settings-row">
                <div className="settings-label-group">
                  <span className="settings-label">Strict Hotkey Modifiers</span>
                  <span className="settings-sublabel">
                    On: hotkey only fires when modifier keys match exactly. Off:
                    extra held modifiers (e.g. Shift while gaming) are ignored.
                  </span>
                </div>
                <div className="settings-seg-group">
                  {["On", "Off"].map((o) => (
                    <button
                      key={o}
                      className={`settings-seg-btn ${(settings.strictHotkeyModifiers ? "On" : "Off") === o ? "active" : ""}`}
                      onClick={() =>
                        update({ strictHotkeyModifiers: o === "On" })
                      }
                    >
                      {o}
                    </button>
                  ))}
                </div>
              </div>

              <div className="settings-divider" />

              <div className="settings-row">
                <div className="settings-label-group">
                  <span className="settings-label">Minimize to Tray</span>
                  <span className="settings-sublabel">
                    Keep the app running in the system tray when the window is
                    closed.
                  </span>
                </div>
                <div className="settings-seg-group">
                  {["On", "Off"].map((o) => (
                    <button
                      key={o}
                      className={`settings-seg-btn ${(settings.minimizeToTray ? "On" : "Off") === o ? "active" : ""}`}
                      onClick={() => update({ minimizeToTray: o === "On" })}
                    >
                      {o}
                    </button>
                  ))}
                </div>
              </div>

              <div className="settings-divider" />

              <div className="settings-row">
                <div className="settings-label-group">
                  <span className="settings-label">1000 CPS Mode</span>
                  <span className="settings-sublabel">
                    Allows setting click speed up to 1000 CPS. High speeds may
                    increase CPU usage and may not be accurate on all systems.
                  </span>
                </div>
                <div className="settings-seg-group">
                  {["On", "Off"].map((o) => (
                    <button
                      key={o}
                      className={`settings-seg-btn ${(settings.highCpsMode ? "On" : "Off") === o ? "active" : ""}`}
                      onClick={() => {
                        if (o === "On") {
                          if (
                            !settings.dismissedWarnings.includes("high-cps")
                          ) {
                            const confirm = window.confirm(
                              "1000 CPS mode allows very high click speeds. This can increase CPU usage and may not be accurate on all systems. Enable anyway?",
                            );
                            if (!confirm) return;
                            update({
                              highCpsMode: true,
                              dismissedWarnings: [
                                ...settings.dismissedWarnings,
                                "high-cps",
                              ],
                            });
                            return;
                          }
                        }
                        update({ highCpsMode: o === "On" });
                      }}
                    >
                      {o}
                    </button>
                  ))}
                </div>
              </div>

              <div className="settings-divider" />

              <div className="settings-row">
                <div className="settings-label-group">
                  <span className="settings-label">Reset All Settings</span>
                  <span className="settings-sublabel">
                    Will reset all input fields and settings to the Defaults.
                  </span>
                </div>
                <button
                  className="settings-btn-danger"
                  onClick={() => set_pending_action("reset-settings")}
                >
                  {resetting ? "Resetting..." : "Reset"}
                </button>
              </div>
            </>
          )}

          {active_tab === "appearance" && (
            <>
              <div className="settings-row">
                <div className="settings-label-group">
                  <span className="settings-label">Theme</span>
                  <span className="settings-sublabel">
                    Choose a theme for the app.
                  </span>
                </div>
                <div className="settings-seg-group">
                  {THEMES.map((t) => (
                    <button
                      key={t.key}
                      className={`settings-seg-btn ${settings.theme === t.key ? "active" : ""}`}
                      onClick={() => update({ theme: t.key })}
                    >
                      {t.label}
                    </button>
                  ))}
                </div>
              </div>
            </>
          )}

          {active_tab === "overlay" && (
            <>
              <div className="settings-row">
                <div className="settings-label-group">
                  <span className="settings-label">Stop Hitbox Overlay</span>
                  <span className="settings-sublabel">
                    Toggles whether the stop hitbox overlay is shown.
                  </span>
                </div>
                <div className="settings-seg-group">
                  {["On", "Off"].map((o) => (
                    <button
                      key={o}
                      className={`settings-seg-btn ${(settings.showStopOverlay ? "On" : "Off") === o ? "active" : ""}`}
                      onClick={() => update({ showStopOverlay: o === "On" })}
                    >
                      {o}
                    </button>
                  ))}
                </div>
              </div>

              <div className="settings-row">
                <div className="settings-label-group">
                  <span className="settings-label">Stop Reason Alert</span>
                  <span className="settings-sublabel">
                    Shows why the clicker stopped in the title bar.
                  </span>
                </div>
                <div className="settings-seg-group">
                  {["On", "Off"].map((o) => (
                    <button
                      key={o}
                      className={`settings-seg-btn ${(settings.showStopReason ? "On" : "Off") === o ? "active" : ""}`}
                      onClick={() => update({ showStopReason: o === "On" })}
                    >
                      {o}
                    </button>
                  ))}
                </div>
              </div>
            </>
          )}

          {active_tab === "stats" && (
            <>
              <div className="settings-row">
                <div className="settings-label-group">
                  <span className="settings-label">Your Usage Data</span>
                  <span className="settings-sublabel">
                    Personal clicker stats, tracked locally on your device.
                  </span>
                </div>
                {has_stats && (
                  <button
                    className="settings-btn-danger"
                    onClick={handle_reset_stats}
                    disabled={resetting_stats}
                  >
                    {resetting_stats ? "Resetting..." : "Reset Stats"}
                  </button>
                )}
              </div>

              {has_stats ? (
                <>
                  <div className="stats-grid">
                    <div className="stats-cell">
                      <span className="stats-cell-label">Total Clicks</span>
                      <span className="stats-cell-value">
                        {format_number(stats.totalClicks)}
                      </span>
                    </div>
                    <div className="stats-cell">
                      <span className="stats-cell-label">Total Time</span>
                      <span className="stats-cell-value">
                        {format_time(stats.totalTimeSecs)}
                      </span>
                    </div>
                    <div className="stats-cell">
                      <span className="stats-cell-label">Sessions</span>
                      <span className="stats-cell-value">
                        {format_number(stats.totalSessions)}
                      </span>
                    </div>
                    <div className="stats-cell">
                      <span className="stats-cell-label">Avg CPU</span>
                      <span className="stats-cell-value">
                        {format_cpu(stats.avgCpu)}
                      </span>
                    </div>
                    <div className="stats-cell">
                      <span className="stats-cell-label">Clicks / Session</span>
                      <span className="stats-cell-value">
                        {format_number(derived?.clicks_per_session ?? 0)}
                      </span>
                    </div>
                    <div className="stats-cell">
                      <span className="stats-cell-label">Avg Session Time</span>
                      <span className="stats-cell-value">
                        {format_time(derived?.time_per_session ?? 0)}
                      </span>
                    </div>
                    <div className="stats-cell">
                      <span className="stats-cell-label">Avg CPS</span>
                      <span className="stats-cell-value">
                        {format_decimal(derived?.cps ?? 0)}
                      </span>
                    </div>
                  </div>
                </>
              ) : (
                <div className="stats-empty">No runs recorded yet</div>
              )}
            </>
          )}

          {active_tab === "about" && (
            <>
              <div className="social-links">
                <span className="settings-label">Support Me</span>
                <div className="social-icons">
                  <a
                    className="social-icon social-icon--kofi"
                    href="#"
                    title="Ko-fi"
                    onClick={(e) => {
                      e.preventDefault();
                      void openUrl("https://ko-fi.com/Z8Z71T8QD4");
                    }}
                  >
                    <img
                      height="28"
                      style={{ border: 0, height: "28px" }}
                      src="https://storage.ko-fi.com/cdn/kofi3.png?v=6"
                      alt="Buy Me a Coffee at ko-fi.com"
                    />
                  </a>

                  <a
                    className="social-icon social-icon--youtube"
                    href="#"
                    title="YouTube"
                    onClick={(e) => {
                      e.preventDefault();
                      void openUrl("https://youtube.com/@Blur009");
                    }}
                  >
                    <svg
                      viewBox="0 0 24 24"
                      fill="currentColor"
                      width="18"
                      height="18"
                    >
                      <path d="M23.498 6.186a3.016 3.016 0 0 0-2.122-2.136C19.505 3.545 12 3.545 12 3.545s-7.505 0-9.377.505A3.017 3.017 0 0 0 .502 6.186C0 8.07 0 12 0 12s0 3.93.502 5.814a3.016 3.016 0 0 0 2.122 2.136c1.871.505 9.376.505 9.376.505s7.505 0 9.377-.505a3.015 3.015 0 0 0 2.122-2.136C24 15.93 24 12 24 12s0-3.93-.502-5.814zM9.545 15.568V8.432L15.818 12l-6.273 3.568z" />
                    </svg>
                  </a>
                  <a
                    className="social-icon social-icon--twitch"
                    href="#"
                    title="Twitch"
                    onClick={(e) => {
                      e.preventDefault();
                      void openUrl("https://twitch.tv/Blur009");
                    }}
                  >
                    <svg
                      viewBox="0 0 24 24"
                      fill="currentColor"
                      width="18"
                      height="18"
                    >
                      <path d="M11.571 4.714h1.715v5.143H11.57zm4.715 0H18v5.143h-1.714zM6 0L1.714 4.286v15.428h5.143V24l4.286-4.286h3.428L22.286 12V0zm14.571 11.143l-3.428 3.428h-3.429l-3 3v-3H6.857V1.714h13.714z" />
                    </svg>
                  </a>
                  <a
                    className="social-icon social-icon--github"
                    href="#"
                    title="GitHub"
                    onClick={(e) => {
                      e.preventDefault();
                      void openUrl("https://github.com/Blur009/Blur-AutoClicker");
                    }}
                  >
                    <svg
                      viewBox="0 0 24 24"
                      fill="currentColor"
                      width="18"
                      height="18"
                    >
                      <path d="M12 .3a12 12 0 0 0-3.8 23.4c.6.1.8-.2.8-.6v-2c-3.3.7-4-1.4-4-1.4-.5-1.3-1.2-1.7-1.2-1.7-1-.7.1-.7.1-.7 1.1.1 1.7 1.2 1.7 1.2 1 .1.8 1.8 3.4 1.2.1-.7.4-1.2.7-1.5-2.7-.3-5.4-1.3-5.4-6a4.7 4.7 0 0 1 1.2-3.2c-.1-.3-.5-1.5.1-3.2 0 0 1-.3 3.3 1.2a11.2 11.2 0 0 1 6.1 0c2.3-1.5 3.3-1.2 3.3-1.2.6 1.7.2 2.9.1 3.2a4.7 4.7 0 0 1 1.2 3.2c0 4.7-2.8 5.7-5.4 6 .4.3.8 1 .8 2.1v3.1c0 .4.2.7.8.6A12 12 0 0 0 12 .3" />
                    </svg>
                  </a>
                </div>
              </div>

              <div className="settings-divider" />

              <div className="settings-row">
                <div className="settings-label-group settings-label-group--inline">
                  <span className="settings-label">Version</span>
                  <span className="settings-value">
                    v{app_info.version} - Ported by Zynix
                  </span>
                </div>
                <div className="settings-row-actions">
                  <button
                    className="settings-btn-secondary changelog-toggle-btn"
                    onClick={() => set_show_changelog((v) => !v)}
                  >
                    <svg
                      className={`changelog-arrow${show_changelog ? " open" : ""}`}
                      width="10"
                      height="10"
                      viewBox="0 0 10 10"
                      fill="none"
                    >
                      <path
                        d="M3 1L7 5L3 9"
                        stroke="currentColor"
                        strokeWidth="1.5"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      />
                    </svg>
                    {show_changelog ? "Hide Changes" : "Show Changes"}
                  </button>
                  <button
                    className="settings-btn-secondary check-update-btn"
                    onClick={onCheckForUpdate}
                    disabled={updateCheckStatus !== "idle"}
                  >
                    {update_button_label}
                  </button>
                </div>
              </div>
              {show_changelog && <ChangelogContent entries={changelogEntries} />}
            </>
          )}
        </div>
        <div
          className={`settings-fade ${at_bottom ? "settings-fade--hidden" : ""}`}
        />
      </div>

      <ConfirmDialog
        open={pending_action === "reset-settings"}
        title="Reset all settings?"
        message="This will restore all settings to their defaults. Your saved stats will not be affected."
        confirmLabel="Reset"
        onConfirm={confirm_reset_settings}
        onCancel={() => set_pending_action(null)}
      />
      <ConfirmDialog
        open={pending_action === "clear-stats"}
        title="Clear all stats?"
        message="Are you sure you want to reset all stats? This cannot be undone."
        confirmLabel="Clear"
        onConfirm={confirm_reset_stats}
        onCancel={() => set_pending_action(null)}
      />
    </div>
  );
}
