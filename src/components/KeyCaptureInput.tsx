import {
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type KeyboardEvent,
  type MouseEvent,
} from "react";
import {
  capture_hotkey,
  format_hotkey_for_display,
  get_keyboard_layout_map,
} from "../hotkeys";
import { isAlphabeticKeyboardKey } from "../keyboardKeyCase";
import type { KeyboardKeyCase, MouseButton } from "../store";

interface Props {
  value: string;
  onChange: (next: string) => void;
  className?: string;
  style?: CSSProperties;
  keyboardKeyCase?: KeyboardKeyCase;
  onMouseButtonCapture?: (button: MouseButton) => void;
}

const MODIFIER_KEYS = new Set(["Control", "Shift", "Alt", "Meta"]);

function applyKeyboardKeyCase(
  value: string,
  displayText: string,
  keyboardKeyCase?: KeyboardKeyCase,
) {
  if (!keyboardKeyCase || !isAlphabeticKeyboardKey(value)) {
    return displayText;
  }

  return keyboardKeyCase === "upper"
    ? displayText.toUpperCase()
    : displayText.toLowerCase();
}

export default function KeyCaptureInput({
  value,
  onChange,
  className,
  style,
  keyboardKeyCase,
  onMouseButtonCapture,
}: Props) {
  const [listening, setListening] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const rightClickStartedWhileListeningRef = useRef(false);
  const [layoutMap, setLayoutMap] =
    useState<Awaited<ReturnType<typeof get_keyboard_layout_map>>>(null);

  useEffect(() => {
    let active = true;
    get_keyboard_layout_map().then((map) => {
      if (active) setLayoutMap(map);
    });
    return () => {
      active = false;
    };
  }, []);

  const displayText = useMemo(() => {
    if (listening) return "Press a key...";
    if (!value) return "Select key";
    return applyKeyboardKeyCase(
      value,
      format_hotkey_for_display(value, layoutMap),
      keyboardKeyCase,
    );
  }, [keyboardKeyCase, layoutMap, listening, value]);

  const handleKeyDown = (event: KeyboardEvent<HTMLInputElement>) => {
    event.preventDefault();
    event.stopPropagation();

    if (event.key === "Escape") {
      setListening(false);
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
      setListening(false);
      event.currentTarget.blur();
      return;
    }

    if (MODIFIER_KEYS.has(event.key)) return;

    const captured = capture_hotkey({
      key: event.key,
      code: event.code,
      location: event.location,
      ctrlKey: false,
      altKey: false,
      shiftKey: false,
      metaKey: false,
    });

    if (captured) {
      const mainKey = captured.split("+").pop() ?? captured;
      onChange(mainKey);
      setListening(false);
      event.currentTarget.blur();
    }
  };

  const handleMouseDown = (event: MouseEvent<HTMLInputElement>) => {
    if (event.button !== 2) return;

    rightClickStartedWhileListeningRef.current = listening;
    if (!listening) {
      event.preventDefault();
    }
  };

  const handleContextMenu = (event: MouseEvent<HTMLInputElement>) => {
    event.preventDefault();
    event.stopPropagation();

    if (rightClickStartedWhileListeningRef.current) {
      onMouseButtonCapture?.("Right");
      setListening(false);
      inputRef.current?.blur();
    } else {
      onChange("");
      setListening(false);
      inputRef.current?.blur();
    }

    rightClickStartedWhileListeningRef.current = false;
  };

  return (
    <input
      ref={inputRef}
      type="text"
      readOnly
      className={className}
      value={displayText}
      onClick={() => setListening(true)}
      onBlur={() => setListening(false)}
      onKeyDown={handleKeyDown}
      onMouseDown={handleMouseDown}
      onContextMenu={handleContextMenu}
      spellCheck={false}
      title="Right click input to clear"
      style={{
        cursor: "pointer",
        textAlign: "center",
        ...style,
      }}
    />
  );
}
