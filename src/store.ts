import { getVersion } from "@tauri-apps/api/app";
import { LazyStore } from "@tauri-apps/plugin-store";

const store = new LazyStore("settings.json");

export const APP_VERSION = await getVersion();

export type SavedPanel = "simple" | "advanced";
export type ExplanationMode = "off" | "text";
export type Theme = "dark" | "oled" | "catppuccin-mocha" | "light" | "nord" | "gruvbox-dark" | "tokyo-night";

export interface Settings {
  version: string;
  clickSpeed: number;
  clickInterval: "s" | "m" | "h" | "d";
  mouseButton: "Left" | "Middle" | "Right";
  hotkey: string;
  mode: "Toggle" | "Hold";
  dutyCycleEnabled: boolean;
  dutyCycle: number;
  speedVariationEnabled: boolean;
  speedVariation: number;
  doubleClickEnabled: boolean;
  doubleClickDelay: number;
  clickLimitEnabled: boolean;
  clickLimit: number;
  timeLimitEnabled: boolean;
  timeLimit: number;
  timeLimitUnit: "s" | "m" | "h";
  cornerStopEnabled: boolean;
  cornerStopTL: number;
  cornerStopTR: number;
  cornerStopBL: number;
  cornerStopBR: number;
  edgeStopEnabled: boolean;
  edgeStopTop: number;
  edgeStopBottom: number;
  edgeStopLeft: number;
  edgeStopRight: number;
  positionEnabled: boolean;
  positionX: number;
  positionY: number;
  disableScreenshots: boolean;
  advancedSettingsEnabled: boolean;
  explanationMode: ExplanationMode;
  lastPanel: SavedPanel;
  showStopReason: boolean;
  showStopOverlay: boolean;
  strictHotkeyModifiers: boolean;
  theme: Theme;
  dismissedWarnings: string[];
}

export interface ClickerStatus {
  running: boolean;
  clickCount: number;
  lastError: string | null;
  stopReason: string | null;
}

export interface AppInfo {
  version: string;
  updateStatus: string;
  screenshotProtectionSupported: boolean;
}

export const DEFAULT_SETTINGS: Settings = {
  version: APP_VERSION,
  clickSpeed: 25,
  clickInterval: "s",
  mouseButton: "Left",
  hotkey: "ctrl+y",
  mode: "Toggle",
  dutyCycleEnabled: true,
  dutyCycle: 45,
  speedVariationEnabled: true,
  speedVariation: 35,
  doubleClickEnabled: false,
  doubleClickDelay: 40,
  clickLimitEnabled: false,
  clickLimit: 1000,
  timeLimitEnabled: false,
  timeLimit: 60,
  timeLimitUnit: "s",
  cornerStopEnabled: true,
  cornerStopTL: 50,
  cornerStopTR: 50,
  cornerStopBL: 50,
  cornerStopBR: 50,
  edgeStopEnabled: true,
  edgeStopTop: 40,
  edgeStopBottom: 40,
  edgeStopLeft: 40,
  edgeStopRight: 40,
  positionEnabled: false,
  positionX: 0,
  positionY: 0,
  disableScreenshots: false,
  advancedSettingsEnabled: true,
  explanationMode: "text",
  lastPanel: "simple",
  showStopReason: true,
  showStopOverlay: true,
  strictHotkeyModifiers: false,
  theme: "dark",
  dismissedWarnings: [],
};

function sanitize_saved_panel(value: unknown): SavedPanel {
  return value === "advanced" ? value : "simple";
}

function sanitize_explanation_mode(
  input: Partial<Settings> | null | undefined,
): ExplanationMode {
  const saved = (input ?? {}) as Partial<Settings> & {
    functionExplanationsEnabled?: boolean;
    toolTipsEnabled?: boolean;
    explanationMode?: unknown;
  };

  if (saved.explanationMode === "off" || saved.explanationMode === "text") {
    return saved.explanationMode;
  }

  if (saved.toolTipsEnabled) return "text";
  if (saved.functionExplanationsEnabled === false) return "off";
  return "text";
}

function sanitize_boolean(value: unknown, fallback: boolean): boolean {
  return typeof value === "boolean" ? value : fallback;
}

function clamp_number(
  value: unknown,
  fallback: number,
  min?: number,
  max?: number,
) {
  const parsed =
    typeof value === "number" && Number.isFinite(value) ? value : fallback;
  const min_clamped = min === undefined ? parsed : Math.max(min, parsed);
  return max === undefined ? min_clamped : Math.min(max, min_clamped);
}

function sanitize_settings(input?: Partial<Settings> | null): Settings {
  const saved = (input ?? {}) as Partial<Settings> & {
    speedVariationMax?: unknown;
    telemetryEnabled?: unknown;
  };
  const legacy_speed_variation = clamp_number(
    saved.speedVariationMax,
    DEFAULT_SETTINGS.speedVariation,
    0,
    200,
  );

  return {
    ...DEFAULT_SETTINGS,
    ...saved,
    version: APP_VERSION,
    clickSpeed: clamp_number(
      saved.clickSpeed,
      DEFAULT_SETTINGS.clickSpeed,
      1,
      500,
    ),
    dutyCycleEnabled: sanitize_boolean(
      saved.dutyCycleEnabled,
      DEFAULT_SETTINGS.dutyCycleEnabled,
    ),
    speedVariationEnabled: sanitize_boolean(
      saved.speedVariationEnabled,
      DEFAULT_SETTINGS.speedVariationEnabled,
    ),
    speedVariation: clamp_number(saved.speedVariation, legacy_speed_variation, 0, 200),
    doubleClickDelay: clamp_number(
      saved.doubleClickDelay,
      DEFAULT_SETTINGS.doubleClickDelay,
      20,
      9999,
    ),
    clickLimit: clamp_number(saved.clickLimit, DEFAULT_SETTINGS.clickLimit, 1),
    timeLimit: clamp_number(saved.timeLimit, DEFAULT_SETTINGS.timeLimit, 1),
    cornerStopTL: clamp_number(
      saved.cornerStopTL,
      DEFAULT_SETTINGS.cornerStopTL,
      0,
      999,
    ),
    cornerStopTR: clamp_number(
      saved.cornerStopTR,
      DEFAULT_SETTINGS.cornerStopTR,
      0,
      999,
    ),
    cornerStopBL: clamp_number(
      saved.cornerStopBL,
      DEFAULT_SETTINGS.cornerStopBL,
      0,
      999,
    ),
    cornerStopBR: clamp_number(
      saved.cornerStopBR,
      DEFAULT_SETTINGS.cornerStopBR,
      0,
      999,
    ),
    edgeStopTop: clamp_number(
      saved.edgeStopTop,
      DEFAULT_SETTINGS.edgeStopTop,
      0,
      999,
    ),
    edgeStopBottom: clamp_number(
      saved.edgeStopBottom,
      DEFAULT_SETTINGS.edgeStopBottom,
      0,
      999,
    ),
    edgeStopLeft: clamp_number(
      saved.edgeStopLeft,
      DEFAULT_SETTINGS.edgeStopLeft,
      0,
      999,
    ),
    edgeStopRight: clamp_number(
      saved.edgeStopRight,
      DEFAULT_SETTINGS.edgeStopRight,
      0,
      999,
    ),
    positionX: clamp_number(saved.positionX, DEFAULT_SETTINGS.positionX, 0),
    positionY: clamp_number(saved.positionY, DEFAULT_SETTINGS.positionY, 0),
    disableScreenshots: false,
    explanationMode: sanitize_explanation_mode(saved),
    lastPanel: sanitize_saved_panel(saved.lastPanel),
    theme: (
      [
        "dark",
        "oled",
        "catppuccin-mocha",
        "light",
        "nord",
        "gruvbox-dark",
        "tokyo-night",
      ] as Theme[]
    ).includes(saved.theme as Theme)
      ? (saved.theme as Theme)
      : "dark",
  };
}

export async function load_settings(): Promise<Settings> {
  const saved = await store.get<Partial<Settings>>("settings");
  return sanitize_settings(saved);
}

export async function save_settings(settings: Settings): Promise<void> {
  await store.set("settings", sanitize_settings(settings));
  await store.save();
}

export async function clear_saved_settings(): Promise<void> {
  await store.set("settings", DEFAULT_SETTINGS);
  await store.save();
}
