import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  currentMonitor,
  getCurrentWindow,
  LogicalSize,
} from "@tauri-apps/api/window";
import { lazy, useEffect, useRef, useState } from "react";
import SystemWarningBanner from "./components/SystemWarningBanner";
import UpdateBanner from "./components/Updatebanner";
import { canonicalize_hotkey_for_backend } from "./hotkeys";
import {
  APP_VERSION,
  DEFAULT_SETTINGS,
  type AppInfo,
  type ClickerStatus,
  type Settings,
  clear_saved_settings,
  load_settings,
  save_settings,
} from "./store";

const SimplePanel = lazy(() => import("./components/panels/SimplePanel"));
const AdvancedPanel = lazy(() => import("./components/panels/AdvancedPanel"));
const SettingsPanel = lazy(() => import("./components/panels/SettingsPanel"));
const TitleBar = lazy(() => import("./components/TitleBar"));
const AdvancedPanelCompact = lazy(
  () => import("./components/panels/AdvancedPanelCompact"),
);

export type Tab = "simple" | "advanced" | "settings";

const BACKEND_SETTINGS_SCHEMA_VERSION = 5;

function get_panel_size(tab: Tab, settings: Settings, banner_count: number) {
  const extra = banner_count * 30;
  if (tab === "settings") return { width: 500, height: 600 + extra };
  if (tab === "simple") return { width: 550, height: 175 + extra };
  return settings.explanationMode === "off"
    ? { width: 600, height: 600 + extra }
    : { width: 800, height: 650 + extra };
}

const text_scale = await invoke<number>("get_text_scale_factor");
await invoke("set_webview_zoom", { factor: 1.0 / text_scale });

async function get_clamped_panel_size(
  size: { width: number; height: number },
  text_scale: number,
) {
  const monitor = await currentMonitor();
  if (!monitor) return size;

  const scale = Math.max(monitor.scaleFactor || 1, 1);

  let work_area_w = Math.floor(monitor.workArea.size.width / scale);
  let work_area_h = Math.floor(monitor.workArea.size.height / scale);
  if (work_area_w < 300 || work_area_h < 300) {
    work_area_w = Math.floor(monitor.size.width / scale);
    work_area_h = Math.floor(monitor.size.height / scale);
  }

  const horizontal_margin = 24;
  const vertical_margin = 24;

  return {
    width: Math.min(
      Math.ceil(size.width * text_scale),
      Math.max(360, work_area_w - horizontal_margin),
    ),
    height: Math.min(
      Math.ceil(size.height * text_scale),
      Math.max(220, work_area_h - vertical_margin),
    ),
  };
}

const DEFAULT_STATUS: ClickerStatus = {
  running: false,
  clickCount: 0,
  lastError: null,
  stopReason: null,
};

const DEFAULT_APP_INFO: AppInfo = {
  version: APP_VERSION,
  updateStatus: "Update checks are disabled in development",
  screenshotProtectionSupported: false,
};

async function sync_settings_to_backend(settings: Settings) {
  await invoke("update_settings", {
    settings: {
      ...settings,
      version: BACKEND_SETTINGS_SCHEMA_VERSION,
    },
  });
}

async function register_hotkey_candidate(hotkey: string) {
  const canonical_hotkey = await canonicalize_hotkey_for_backend(hotkey);
  return invoke<string>("register_hotkey", { hotkey: canonical_hotkey });
}

function wait(ms: number) {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

interface SystemDepsInfo {
  display_server: string;
  uinput_accessible: boolean;
  in_input_group: boolean;
  uinput_module_loaded: boolean;
  is_root: boolean;
  warnings: string[];
}

function warning_id(msg: string): string {
  if (msg.startsWith("XWayland detected")) return "xwayland";
  if (msg.startsWith("uinput module not loaded")) return "uinput-module";
  if (msg.startsWith("User is not in the 'input' group")) return "input-group";
  if (msg.startsWith("/dev/uinput exists but is not accessible")) return "uinput-access";
  if (msg.startsWith("X11 connection failed")) return "x11-connection";
  let h = 0;
  for (let i = 0; i < msg.length; i++) {
    h = ((h << 5) - h + msg.charCodeAt(i)) | 0;
  }
  return `w${h.toString(16)}`;
}

export default function App() {
  const [tab, set_tab] = useState<Tab>("simple");
  const [settings, set_settings] = useState<Settings>(DEFAULT_SETTINGS);
  const [settings_loaded, set_settings_loaded] = useState(false);
  const [status, set_status] = useState<ClickerStatus>(DEFAULT_STATUS);
  const [app_info, set_app_info] = useState<AppInfo>(DEFAULT_APP_INFO);
  const [update_info, set_update_info] = useState<{
    currentVersion: string;
    latestVersion: string;
  } | null>(null);
  const [system_warnings, set_system_warnings] = useState<string[]>([]);

  const hotkey_timer = useRef<number | null>(null);
  const hotkey_request_id_ref = useRef(0);
  const ui_settings_ref = useRef<Settings>(DEFAULT_SETTINGS);
  const committed_settings_ref = useRef<Settings>(DEFAULT_SETTINGS);
  const last_valid_hotkey_ref = useRef(DEFAULT_SETTINGS.hotkey);
  const launch_window_placement_done = useRef(false);
  const save_timer_ref = useRef<ReturnType<typeof setTimeout> | null>(null);
  const resize_timeout = useRef<ReturnType<typeof setTimeout> | null>(null);
  const last_resize_time = useRef(0);
  const last_tab_ref = useRef<Tab | null>(null);

  const set_ui_settings = (next_settings: Settings) => {
    ui_settings_ref.current = next_settings;
    set_settings(next_settings);
  };

  const schedule_save = (next_settings: Settings) => {
    if (save_timer_ref.current) {
      clearTimeout(save_timer_ref.current);
    }
    save_timer_ref.current = setTimeout(() => {
      save_settings(next_settings).catch((err) => {
        console.error("Failed to save settings:", err);
      });
    }, 100);
  };

  const persist_committed_settings = (
    next_committed_settings: Settings,
    next_ui_settings: Settings,
  ) => {
    committed_settings_ref.current = next_committed_settings;
    set_ui_settings(next_ui_settings);

    if (!settings_loaded) {
      return;
    }

    sync_settings_to_backend(next_committed_settings).catch((err) => {
      console.error("Failed to sync settings:", err);
    });
    schedule_save(next_committed_settings);
  };

  const restore_last_valid_hotkey = () => {
    const restored_hotkey = last_valid_hotkey_ref.current;
    if (ui_settings_ref.current.hotkey === restored_hotkey) {
      return;
    }

    set_ui_settings({
      ...ui_settings_ref.current,
      hotkey: restored_hotkey,
    });
  };

  const queue_hotkey_registration = (hotkey: string) => {
    if (!settings_loaded) {
      return;
    }

    if (hotkey_timer.current !== null) {
      window.clearTimeout(hotkey_timer.current);
    }

    const request_id = ++hotkey_request_id_ref.current;
    hotkey_timer.current = window.setTimeout(() => {
      hotkey_timer.current = null;

      register_hotkey_candidate(hotkey)
        .then((normalized_hotkey) => {
          if (hotkey_request_id_ref.current !== request_id) {
            return;
          }

          last_valid_hotkey_ref.current = normalized_hotkey;
          const next_committed_settings = {
            ...committed_settings_ref.current,
            hotkey: normalized_hotkey,
          };
          const next_ui_settings = {
            ...ui_settings_ref.current,
            hotkey: normalized_hotkey,
          };

          persist_committed_settings(next_committed_settings, next_ui_settings);
        })
        .catch((err) => {
          if (hotkey_request_id_ref.current !== request_id) {
            return;
          }

          console.error("Failed to register hotkey:", err);
          restore_last_valid_hotkey();
        });
    }, 250);
  };

  const update_settings = (patch: Partial<Settings>) => {
    const { hotkey, ...rest } = patch;

    if (Object.keys(rest).length > 0) {
      const next_ui_settings = { ...ui_settings_ref.current, ...rest };
      const next_committed_settings = { ...committed_settings_ref.current, ...rest };
      persist_committed_settings(next_committed_settings, next_ui_settings);
    }

    if (hotkey !== undefined) {
      set_ui_settings({
        ...ui_settings_ref.current,
        hotkey,
      });
      queue_hotkey_registration(hotkey);
    }
  };

  const apply_startup_window_placement = async () => {
    const monitor = await currentMonitor().catch(() => null);
    if (!monitor) return;
    await getCurrentWindow().center().catch(() => {});
  };

  const handle_window_close = async () => {
    await getCurrentWindow().close();
  };

  useEffect(() => {
    let mounted = true;

    void Promise.all([
      load_settings(),
      invoke<AppInfo>("get_app_info"),
      invoke<ClickerStatus>("get_status"),
      invoke<SystemDepsInfo>("check_system_deps"),
    ])
      .then(async ([loaded_settings, loaded_app_info, loaded_status, system_deps]) => {
        if (!mounted) return;

        if (system_deps.warnings.length > 0) {
          const active = system_deps.warnings.filter(
            (w) => !loaded_settings.dismissedWarnings.includes(warning_id(w)),
          );
          set_system_warnings(active);
        }

        let registered_hotkey = loaded_settings.hotkey;
        try {
          registered_hotkey = await register_hotkey_candidate(loaded_settings.hotkey);
        } catch (err) {
          console.error("Failed to register saved hotkey:", err);
          registered_hotkey = last_valid_hotkey_ref.current;
        }

        const hydrated_settings =
          registered_hotkey !== loaded_settings.hotkey
            ? { ...loaded_settings, hotkey: registered_hotkey }
            : loaded_settings;

        last_valid_hotkey_ref.current = hydrated_settings.hotkey;
        ui_settings_ref.current = hydrated_settings;
        committed_settings_ref.current = hydrated_settings;

        set_tab(hydrated_settings.lastPanel);
        set_settings(hydrated_settings);
        set_app_info(loaded_app_info);
        set_status(loaded_status);
        set_settings_loaded(true);

        await sync_settings_to_backend(hydrated_settings);

        if (hydrated_settings.hotkey !== loaded_settings.hotkey) {
          await save_settings(hydrated_settings);
        }
      })
      .catch((err) => {
        console.error("Failed to boot app:", err);
        if (!mounted) return;
        set_settings_loaded(true);
      });

    return () => {
      mounted = false;
      if (hotkey_timer.current !== null) {
        window.clearTimeout(hotkey_timer.current);
      }
      if (save_timer_ref.current) {
        clearTimeout(save_timer_ref.current);
      }
      if (resize_timeout.current) {
        clearTimeout(resize_timeout.current);
      }
    };
  }, []);

  useEffect(() => {
    let cleanup: (() => void) | undefined;

    listen<ClickerStatus>("clicker-status", (event) => {
      set_status(event.payload);
    })
      .then((unlisten) => {
        cleanup = unlisten;
      })
      .catch((err) => {
        console.error("Failed to listen for clicker status:", err);
      });

    return () => {
      cleanup?.();
    };
  }, []);

  useEffect(() => {
    if (resize_timeout.current) {
      clearTimeout(resize_timeout.current);
      resize_timeout.current = null;
    }

    const root = document.querySelector(".app-root") as HTMLElement;

    void (async () => {
      try {
        const text_scale = await invoke<number>("get_text_scale_factor");
        document.documentElement.style.fontSize = `${16 * text_scale}px`;
        console.log("Windows Text Scale:", text_scale);
        console.log(
          "Actual Root Font Size:",
          getComputedStyle(document.documentElement).fontSize,
        );

        const banner_count = (update_info ? 1 : 0) + system_warnings.length;
        const preferred_size = get_panel_size(tab, settings, banner_count);
        const { width, height } = await get_clamped_panel_size(
          preferred_size,
          text_scale,
        );

        const app_window = getCurrentWindow();

        last_resize_time.current = Date.now();

        if (!launch_window_placement_done.current) {
          await app_window.setSize(new LogicalSize(width, height));
          root.style.width = `${width}px`;
          root.style.height = `${height}px`;
          await wait(30);
          await apply_startup_window_placement();
          launch_window_placement_done.current = true;
          last_tab_ref.current = tab;
          return;
        }

        const current_size = await app_window.innerSize();
        const monitor_scale = await app_window.scaleFactor();
        const current_h = current_size.height / monitor_scale;
        const current_w = current_size.width / monitor_scale;

        const is_tab_change = tab !== last_tab_ref.current;
        last_tab_ref.current = tab;

        if (is_tab_change) {
          if (width < current_w || height < current_h) {
            const snap_w = width >= current_w ? width : current_w;
            const snap_h = height >= current_h ? height : current_h;

            if (snap_w !== current_w || snap_h !== current_h) {
              await app_window.setSize(new LogicalSize(snap_w, snap_h));
            }

            root.style.width = `${width}px`;
            root.style.height = `${height}px`;

            resize_timeout.current = setTimeout(async () => {
              await app_window.setSize(new LogicalSize(width, height));
              resize_timeout.current = null;
            }, 320);
          } else {
            await app_window.setSize(new LogicalSize(width, height));
            root.style.width = `${current_w}px`;
            root.style.height = `${current_h}px`;

            void root.offsetHeight;

            root.style.width = `${width}px`;
            root.style.height = `${height}px`;
          }
        } else {
          if (width > current_w || height > current_h) {
            const grow_w = Math.max(width, current_w);
            const grow_h = Math.max(height, current_h);
            await app_window.setSize(new LogicalSize(grow_w, grow_h));
            root.style.width = `${grow_w}px`;
            root.style.height = `${grow_h}px`;
          }
        }
      } catch (err) {
        console.error("Failed to size window:", err);
      }
    })();
  }, [settings, settings_loaded, tab, update_info, system_warnings]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    const app_window = getCurrentWindow();
    app_window
      .onResized(() => {
        if (Date.now() - last_resize_time.current < 500) return;
        const root = document.querySelector(".app-root") as HTMLElement | null;
        if (root && (root.style.width || root.style.height)) {
          const saved_transition = root.style.transition;
          root.style.transition = "none";
          root.style.width = "";
          root.style.height = "";
          void root.offsetHeight;
          root.style.transition = saved_transition;
        }
      })
      .then((fn) => {
        unlisten = fn;
      })
      .catch((err) => {
        console.error("Failed to listen for resize:", err);
      });

    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    const check_for_updates = () => {
      invoke<{
        currentVersion: string;
        latestVersion: string;
        updateAvailable: boolean;
      }>("check_for_updates")
        .then((result) => {
          if (result?.updateAvailable) {
            set_update_info({
              currentVersion: result.currentVersion,
              latestVersion: result.latestVersion,
            });
          }
        })
        .catch((err) => console.error("Update check failed:", err));
    };

    check_for_updates();
    const interval = setInterval(check_for_updates, 60 * 60 * 1000);
    return () => clearInterval(interval);
  }, []);

  useEffect(() => {
    document.documentElement.dataset.theme = settings.theme ?? "dark";
  }, [settings.theme]);

  const handle_tab_change = (next_tab: Tab) => {
    set_tab(next_tab);

    if (next_tab === "settings") return;
    if (committed_settings_ref.current.lastPanel === next_tab) return;

    update_settings({
      lastPanel: next_tab,
    });
  };

  const handle_reset_settings = async () => {
    try {
      if (hotkey_timer.current !== null) {
        window.clearTimeout(hotkey_timer.current);
        hotkey_timer.current = null;
      }
      hotkey_request_id_ref.current += 1;

      await invoke("reset_settings");
      await clear_saved_settings();

      last_valid_hotkey_ref.current = DEFAULT_SETTINGS.hotkey;
      committed_settings_ref.current = DEFAULT_SETTINGS;
      ui_settings_ref.current = DEFAULT_SETTINGS;

      set_settings(DEFAULT_SETTINGS);
      set_tab("simple");
      launch_window_placement_done.current = false;
    } catch (err) {
      console.error("Failed to reset settings:", err);
    }
  };

  const handle_pick_position = async () => {
    try {
      const point = await invoke<{ x: number; y: number }>("pick_position");
      update_settings({
        positionEnabled: true,
        positionX: point.x,
        positionY: point.y,
      });
    } catch (err) {
      console.error("Failed to pick position:", err);
    }
  };

  return (
    <div className="app-root" data-tab={tab}>
      <TitleBar
        tab={tab}
        set_tab={handle_tab_change}
        running={status.running}
        stopReason={
          settings.showStopReason && tab === "advanced"
            ? status.stopReason
            : null
        }
        on_request_close={handle_window_close}
      />
      {system_warnings.map((msg) => (
        <SystemWarningBanner
          key={warning_id(msg)}
          message={msg}
          on_dismiss={() => {
            set_system_warnings((prev) => prev.filter((w) => w !== msg));
            const next = Array.from(
              new Set([
                ...(ui_settings_ref.current.dismissedWarnings ?? []),
                warning_id(msg),
              ]),
            );
            update_settings({ dismissedWarnings: next });
          }}
        />
      ))}
      {update_info && (
        <UpdateBanner
          key={`${update_info.currentVersion}:${update_info.latestVersion}`}
          currentVersion={update_info.currentVersion}
          latestVersion={update_info.latestVersion}
        />
      )}
      <div
        className="resize-handle"
        onMouseDown={() => {
          getCurrentWindow()
            .startResizeDragging("SouthEast")
            .catch((err) => console.error("Resize drag failed:", err));
        }}
        title="Resize"
      />
      <main className="panel-area">
        {tab === "simple" && (
          <SimplePanel settings={settings} update={update_settings} />
        )}
        {tab === "advanced" &&
          (settings.explanationMode === "off" ? (
            <AdvancedPanelCompact
              settings={settings}
              update={update_settings}
              on_pick_position={handle_pick_position}
            />
          ) : (
            <AdvancedPanel
              settings={settings}
              update={update_settings}
              on_pick_position={handle_pick_position}
            />
          ))}
        {tab === "settings" && (
          <SettingsPanel
            settings={settings}
            update={update_settings}
            app_info={app_info}
            onReset={handle_reset_settings}
          />
        )}
      </main>
    </div>
  );
}
