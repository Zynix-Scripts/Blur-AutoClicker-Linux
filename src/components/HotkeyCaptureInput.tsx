import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  capture_hotkey,
  capture_mouse_hotkey,
  capture_wheel_hotkey,
  format_hotkey_for_display,
  get_keyboard_layout_map,
} from "../hotkeys";
import { useTranslation, type TranslationKey } from "../i18n";

interface Props {
  value: string;
  onChange: (next: string) => void;
  className: string;
  style?: React.CSSProperties;
}

export default function HotkeyCaptureInput({
  value,
  onChange,
  className,
  style,
}: Props) {
  const [listening, set_listening] = useState(false);
  const [layout_map, set_layout_map] =
    useState<Awaited<ReturnType<typeof get_keyboard_layout_map>>>(null);

  useEffect(() => {
    let active = true;

    get_keyboard_layout_map().then((map) => {
      if (active) {
        set_layout_map(map);
      }
    });

    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    return () => {
      if (suppressResetTimerRef.current !== null) {
        window.clearTimeout(suppressResetTimerRef.current);
      }
    };
  }, []);

  useEffect(() => {
    invoke("set_hotkey_capture_active", { active: listening }).catch((err) => {
      console.error("Failed to toggle hotkey capture state:", err);
    });

    return () => {
      if (!listening) return;

      invoke("set_hotkey_capture_active", { active: false }).catch((err) => {
        console.error("Failed to clear hotkey capture state:", err);
      });
    };
  }, [listening]);

  const display_text = useMemo(
    () =>
      listening ? "Press keys..." : format_hotkey_for_display(value, layout_map),
    [layout_map, listening, value],
  );

  const accept_hotkey = (
    next_hotkey: string | null,
    target: HTMLInputElement,
  ) => {
    if (!next_hotkey) return;
    onChange(next_hotkey);
    set_listening(false);
    target.blur();
  };

  const handle_key_down = (event: React.KeyboardEvent<HTMLInputElement>) => {
    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      set_listening(false);
      event.currentTarget.blur();
      return;
    }

    if (
      (event.key === "Backspace" || event.key === "Delete") &&
      !event.ctrlKey &&
      !event.altKey &&
      !event.shiftKey &&
      !event.metaKey
    ) {
      onChange("");
      set_listening(false);
      event.currentTarget.blur();
      return;
    }

    accept_hotkey(
      capture_hotkey({
        key: event.key,
        code: event.code,
        location: event.location,
        ctrlKey: event.ctrlKey,
        altKey: event.altKey,
        shiftKey: event.shiftKey,
        metaKey: event.metaKey,
      }),
      event.currentTarget,
    );
  };

  const handle_mouse_down = (event: React.MouseEvent<HTMLInputElement>) => {


    if (!listening) return;

    if (event.button === 0) {
      const has_modifier =
        event.ctrlKey || event.altKey || event.shiftKey || event.metaKey;
      if (!has_modifier) return;
    }

    event.preventDefault();
    event.stopPropagation();
    accept_hotkey(capture_mouse_hotkey(event), event.currentTarget);
  };

  const handle_wheel = (event: React.WheelEvent<HTMLInputElement>) => {
    if (!listening) return;

    event.preventDefault();
    event.stopPropagation();
    accept_hotkey(capture_wheel_hotkey(event), event.currentTarget);
  };

  const handle_context_menu = (event: React.MouseEvent<HTMLInputElement>) => {

    if (listening) {
      event.preventDefault();
      event.stopPropagation();

      if (event.key === "Escape") {
        finishCapture();
        return;
      }

      if (
        (event.key === "Backspace" || event.key === "Delete") &&
        !event.ctrlKey &&
        !event.altKey &&
        !event.shiftKey &&
        !event.metaKey
      ) {
        finishCapture("");
        return;
      }

      const nextHotkey = captureHotkey(event);
      if (!nextHotkey) return;

      finishCapture(nextHotkey);
    };

    const handleMouseDown = (event: MouseEvent) => {
      const input = inputRef.current;
      const isInputTarget =
        input !== null &&
        event.target instanceof Node &&
        input.contains(event.target);

      if (
        isInputTarget &&
        event.button === 0 &&
        performance.now() < ignorePrimaryInputMouseUntilRef.current
      ) {
        return;
      }

      const nextHotkey = captureMouseHotkey(event);
      if (!nextHotkey) return;

      suppressedMouseButtonRef.current = event.button;
      if (suppressResetTimerRef.current !== null) {
        window.clearTimeout(suppressResetTimerRef.current);
      }
      suppressResetTimerRef.current = window.setTimeout(() => {
        suppressedMouseButtonRef.current = null;
        suppressResetTimerRef.current = null;
      }, 200);

      if (event.cancelable) {
        event.preventDefault();
      }
      event.stopPropagation();

      finishCapture(nextHotkey);
    };

    window.addEventListener("keydown", handleKeyDown, true);
    window.addEventListener("mousedown", handleMouseDown, true);

    return () => {
      window.removeEventListener("keydown", handleKeyDown, true);
      window.removeEventListener("mousedown", handleMouseDown, true);
    };
  }, [listening, onChange]);

  const hotkeyLabels = useMemo<HotkeyDisplayLabels>(() => {
    const keyCodes = [
      "up",
      "down",
      "left",
      "right",
      "pageup",
      "pagedown",
      "backspace",
      "delete",
      "insert",
      "home",
      "end",
      "enter",
      "tab",
      "space",
      "escape",
      "esc",
      "capslock",
      "numlock",
      "scrolllock",
      "printscreen",
      "pause",
      "menu",
      "mouseleft",
      "mouseright",
      "mousemiddle",
      "mouse4",
      "mouse5",
      "numpad0",
      "numpad1",
      "numpad2",
      "numpad3",
      "numpad4",
      "numpad5",
      "numpad6",
      "numpad7",
      "numpad8",
      "numpad9",
      "numpadadd",
      "numpadsubtract",
      "numpadmultiply",
      "numpaddivide",
      "numpaddecimal",
    ] as const;

    return {
      empty: t("hotkey.empty"),
      modifiers: {
        ctrl: t("hotkey.modifier.ctrl"),
        alt: t("hotkey.modifier.alt"),
        shift: t("hotkey.modifier.shift"),
        super: t("hotkey.modifier.super"),
      },
      keys: Object.fromEntries(
        keyCodes.map((code) => [code, t(`hotkey.key.${code}` as TranslationKey)]),
      ),
    };
  }, [t]);

  const displayText = useMemo(
    () =>
      listening
        ? t("hotkey.pressKeys")
        : formatHotkeyForDisplay(value, layoutMap, hotkeyLabels),
    [hotkeyLabels, layoutMap, listening, t, value],
  );

  return (
    <input
      ref={inputRef}
      type="text"
      className={className}
      value={display_text}
      readOnly
      onFocus={() => set_listening(true)}
      onBlur={() => set_listening(false)}
      onKeyDown={handle_key_down}
      onMouseDown={handle_mouse_down}
      onWheel={handle_wheel}
      onContextMenu={handle_context_menu}
      spellCheck={false}
      style={style}
    />
  );
}
