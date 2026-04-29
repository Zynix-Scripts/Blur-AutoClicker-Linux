const MODIFIER_ALIASES: Record<string, string> = {
  control: "ctrl",
  ctrl: "ctrl",
  option: "alt",
  alt: "alt",
  shift: "shift",
  meta: "super",
  command: "super",
  cmd: "super",
  super: "super",
  win: "super",
};

const MODIFIER_KEYS = new Set([
  "control",
  "ctrl",
  "shift",
  "alt",
  "meta",
  "os",
  "altgraph",
]);

const SHIFTED_SYMBOL_BASE_MAP: Record<string, string> = {
  "?": "/",
  ":": ";",
  "\"": "'",
  "{": "[",
  "}": "]",
  "|": "\\",
  "+": "=",
  "_": "-",
  "~": "`",
  ">": "<",
};

const NUMPAD_CODE_MAP: Record<string, string> = {
  Numpad0: "numpad0",
  Numpad1: "numpad1",
  Numpad2: "numpad2",
  Numpad3: "numpad3",
  Numpad4: "numpad4",
  Numpad5: "numpad5",
  Numpad6: "numpad6",
  Numpad7: "numpad7",
  Numpad8: "numpad8",
  Numpad9: "numpad9",
  NumpadAdd: "numpadadd",
  NumpadSubtract: "numpadsubtract",
  NumpadMultiply: "numpadmultiply",
  NumpadDivide: "numpaddivide",
  NumpadDecimal: "numpaddecimal",
  NumpadEnter: "numpadenter",
};

const NUMPAD_LOCATION_KEY_MAP: Record<string, string> = {
  "0": "numpad0",
  "1": "numpad1",
  "2": "numpad2",
  "3": "numpad3",
  "4": "numpad4",
  "5": "numpad5",
  "6": "numpad6",
  "7": "numpad7",
  "8": "numpad8",
  "9": "numpad9",
  "+": "numpadadd",
  "-": "numpadsubtract",
  "*": "numpadmultiply",
  "/": "numpaddivide",
  ".": "numpaddecimal",
  enter: "numpadenter",
};

type LayoutMapLike = {
  get(code: string): string | undefined;
};

let layout_map_promise: Promise<LayoutMapLike | null> | null = null;

function normalize_modifier_token(token: string): string | null {
  return MODIFIER_ALIASES[token.trim().toLowerCase()] ?? null;
}

function normalize_named_key(key: string): string | null {
  const lower = key.toLowerCase();

  const key_map: Record<string, string> = {
    enter: "enter",
    tab: "tab",
    spacebar: "space",
    backspace: "backspace",
    delete: "delete",
    insert: "insert",
    home: "home",
    end: "end",
    pageup: "pageup",
    pagedown: "pagedown",
    arrowup: "up",
    arrowdown: "down",
    arrowleft: "left",
    arrowright: "right",
    mouseleft: "mouseleft",
    mouse1: "mouseleft",
    mouseright: "mouseright",
    mouse2: "mouseright",
    mousemiddle: "mousemiddle",
    mouse3: "mousemiddle",
    scrollbutton: "mousemiddle",
    middleclick: "mousemiddle",
    mouse4: "mouse4",
    mouseback: "mouse4",
    xbutton1: "mouse4",
    mouse5: "mouse5",
    mouseforward: "mouse5",
    xbutton2: "mouse5",
    scrollup: "scrollup",
    wheelup: "scrollup",
    scrolldown: "scrolldown",
    wheeldown: "scrolldown",
    numpad0: "numpad0",
    numpad1: "numpad1",
    numpad2: "numpad2",
    numpad3: "numpad3",
    numpad4: "numpad4",
    numpad5: "numpad5",
    numpad6: "numpad6",
    numpad7: "numpad7",
    numpad8: "numpad8",
    numpad9: "numpad9",
    numpadadd: "numpadadd",
    numpadsubtract: "numpadsubtract",
    numpadmultiply: "numpadmultiply",
    numpaddivide: "numpaddivide",
    numpaddecimal: "numpaddecimal",
    numpadenter: "numpadenter",
  };

  if (/^f\d{1,2}$/i.test(key)) {
    return lower;
  }

  return key_map[lower] ?? null;
}

function normalize_numpad_from_code(
  code: string | undefined,
  key: string,
  location: number | undefined,
): string | null {
  if (code && NUMPAD_CODE_MAP[code]) {
    return NUMPAD_CODE_MAP[code];
  }

  if (location !== 3) {
    return null;
  }

  return NUMPAD_LOCATION_KEY_MAP[key.toLowerCase()] ?? null;
}

function display_token_from_stored_value(
  token: string,
  layout_map: LayoutMapLike | null,
): string {
  const trimmed = token.trim();
  if (!trimmed) return trimmed;

  if (trimmed === "IntlBackslash") {
    return layout_map?.get("IntlBackslash") ?? "<";
  }

  if (/^Key[A-Z]$/.test(trimmed)) {
    const mapped = layout_map?.get(trimmed);
    if (mapped) return mapped;
    return trimmed.slice(3).toLowerCase();
  }

  if (/^Digit[0-9]$/.test(trimmed)) {
    return trimmed.slice(5);
  }

  if (NUMPAD_CODE_MAP[trimmed]) {
    return display_token_from_stored_value(NUMPAD_CODE_MAP[trimmed], layout_map);
  }

  const lower = trimmed.toLowerCase();
  const named_display_map: Record<string, string> = {
    up: "Up",
    down: "Down",
    left: "Left",
    right: "Right",
    pageup: "Page Up",
    pagedown: "Page Down",
    backspace: "Backspace",
    delete: "Delete",
    insert: "Insert",
    home: "Home",
    end: "End",
    enter: "Enter",
    tab: "Tab",
    space: "Space",
    escape: "Esc",
    esc: "Esc",
    mouseleft: "Mouse Left",
    mouseright: "Mouse Right",
    mousemiddle: "Scroll Button",
    mouse4: "Mouse Back",
    mouse5: "Mouse Forward",
    scrollup: "Scroll Up",
    scrolldown: "Scroll Down",
    numpad0: "Num 0",
    numpad1: "Num 1",
    numpad2: "Num 2",
    numpad3: "Num 3",
    numpad4: "Num 4",
    numpad5: "Num 5",
    numpad6: "Num 6",
    numpad7: "Num 7",
    numpad8: "Num 8",
    numpad9: "Num 9",
    numpadadd: "Num +",
    numpadsubtract: "Num -",
    numpadmultiply: "Num *",
    numpaddivide: "Num /",
    numpaddecimal: "Num .",
    numpadenter: "Num Enter",
  };

  if (named_display_map[lower]) {
    return named_display_map[lower];
  }

  return trimmed;
}

function normalize_stored_main_key(
  token: string,
  layout_map: LayoutMapLike | null,
): string {
  const trimmed = token.trim();
  if (!trimmed) return trimmed;

  if (trimmed === "IntlBackslash") {
    return "IntlBackslash";
  }

  if (/^Key[A-Z]$/.test(trimmed)) {
    const mapped = layout_map?.get(trimmed);
    return mapped ? mapped.toLowerCase() : trimmed.slice(3).toLowerCase();
  }

  if (/^Digit[0-9]$/.test(trimmed)) {
    return trimmed.slice(5);
  }

  if (NUMPAD_CODE_MAP[trimmed]) {
    return NUMPAD_CODE_MAP[trimmed];
  }

  const lower = trimmed.toLowerCase();
  if (normalize_named_key(lower)?.startsWith("numpad")) {
    return normalize_named_key(lower)!;
  }

  if (lower === "<" || lower === ">") {
    return "IntlBackslash";
  }

  if (SHIFTED_SYMBOL_BASE_MAP[trimmed]) {
    return SHIFTED_SYMBOL_BASE_MAP[trimmed];
  }

  return normalize_named_key(trimmed) ?? lower;
}

export async function get_keyboard_layout_map(): Promise<LayoutMapLike | null> {
  if (!layout_map_promise) {
    const keyboard = (navigator as Navigator & {
      keyboard?: { getLayoutMap?: () => Promise<LayoutMapLike> };
    }).keyboard;

    layout_map_promise = keyboard?.getLayoutMap
      ? keyboard.getLayoutMap().catch(() => null)
      : Promise.resolve(null);
  }

  return layout_map_promise;
}

export async function canonicalize_hotkey_for_backend(value: string): Promise<string> {
  const layout_map = await get_keyboard_layout_map();
  return canonicalize_hotkey_string(value, layout_map);
}

export function capture_hotkey(event: {
  key: string;
  code?: string;
  location?: number;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}): string | null {
  const lower = event.key.toLowerCase();
  if (MODIFIER_KEYS.has(lower) || lower === "escape") {
    return null;
  }

  const main_key =
    normalize_numpad_from_code(event.code, event.key, event.location) ??
    (event.key === " " ? "space" : null) ??
    normalize_named_key(event.key) ??
    (SHIFTED_SYMBOL_BASE_MAP[event.key] ?? (event.key.length === 1 ? lower : null));

  if (!main_key) {
    return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey) parts.push("ctrl");
  if (event.altKey) parts.push("alt");
  if (event.shiftKey) parts.push("shift");
  if (event.metaKey) parts.push("super");
  parts.push(main_key);
  return parts.join("+");
}

export function capture_mouse_hotkey(
  event: {
    button: number;
    ctrlKey: boolean;
    altKey: boolean;
    shiftKey: boolean;
    metaKey: boolean;
  },
  clicker_mouse_button?: string,
): string | null {
  const mouse_map: Record<number, string> = {
    0: "mouseleft",
    1: "mousemiddle",
    2: "mouseright",
    3: "mouse4",
    4: "mouse5",
  };

  const main_key = mouse_map[event.button];
  if (!main_key) return null;

  if (clicker_mouse_button === "Left" && main_key === "mouseleft") return null;
  if (clicker_mouse_button === "Middle" && main_key === "mousemiddle") return null;
  if (clicker_mouse_button === "Right" && main_key === "mouseright") return null;

  if (event.button === 0) {
    const has_modifier =
      event.ctrlKey || event.altKey || event.shiftKey || event.metaKey;
    if (!has_modifier) return null;
  }

  const parts: string[] = [];
  if (event.ctrlKey) parts.push("ctrl");
  if (event.altKey) parts.push("alt");
  if (event.shiftKey) parts.push("shift");
  if (event.metaKey) parts.push("super");
  parts.push(main_key);
  return parts.join("+");
}

export function capture_wheel_hotkey(event: {
  deltaY: number;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}): string | null {
  if (event.deltaY === 0) return null;

  const main_key = event.deltaY < 0 ? "scrollup" : "scrolldown";

  const parts: string[] = [];
  if (event.ctrlKey) parts.push("ctrl");
  if (event.altKey) parts.push("alt");
  if (event.shiftKey) parts.push("shift");
  if (event.metaKey) parts.push("super");
  parts.push(main_key);
  return parts.join("+");
}

export function format_hotkey_for_display(
  value: string,
  layout_map: LayoutMapLike | null,
): string {
  if (!value) return "Click and press keys";

  return value
    .split("+")
    .map((part) => {
      const modifier = normalize_modifier_token(part);
      if (modifier) {
        if (modifier === "ctrl") return "Ctrl";
        if (modifier === "alt") return "Alt";
        if (modifier === "shift") return "Shift";
        return "Super";
      }

      const display = display_token_from_stored_value(part, layout_map);
      return display.length === 1 ? display.toUpperCase() : display;
    })
    .join(" + ");
}

function canonicalize_hotkey_string(
  value: string,
  layout_map: LayoutMapLike | null,
): string {
  const parts: string[] = [];
  let main_key: string | null = null;

  for (const raw_part of value.split("+")) {
    const part = raw_part.trim();
    if (!part) continue;

    const modifier = normalize_modifier_token(part);
    if (modifier) {
      if (!parts.includes(modifier)) {
        parts.push(modifier);
      }
      continue;
    }

    main_key = normalize_stored_main_key(part, layout_map);
  }

  if (main_key) {
    parts.push(main_key);
  }

  return parts.join("+");
}
