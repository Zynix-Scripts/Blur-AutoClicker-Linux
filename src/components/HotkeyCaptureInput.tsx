import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  capture_hotkey,
  capture_mouse_hotkey,
  capture_wheel_hotkey,
  format_hotkey_for_display,
  get_keyboard_layout_map,
} from "../hotkeys";

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
    }
  };

  return (
    <input
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
