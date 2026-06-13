import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { SequencePoint, Settings } from "../../../store";
import "./SequenceSection.css";

interface SequenceSectionProps {
  settings: Settings;
  update: (patch: Partial<Settings>) => void;
  activeSequenceIndex: number | null;
}

function clampInt(value: string, min: number, max: number): number {
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) return min;
  return Math.min(Math.max(parsed, min), max);
}

export default function SequenceSection({
  settings,
  update,
  activeSequenceIndex,
}: SequenceSectionProps) {
  const [picking, setPicking] = useState(false);
  const points = settings.sequencePoints;

  useEffect(() => {
    return () => {
      if (picking) {
        invoke("stop_sequence_pick").catch(console.error);
      }
    };
  }, [picking]);

  const toggleEnabled = () => {
    update({ sequenceEnabled: !settings.sequenceEnabled });
  };

  const startPick = async () => {
    try {
      await invoke("start_sequence_pick");
      setPicking(true);
    } catch (e) {
      console.error("Failed to start sequence pick:", e);
    }
  };

  const stopPick = async () => {
    try {
      await invoke("stop_sequence_pick");
      setPicking(false);
    } catch (e) {
      console.error("Failed to stop sequence pick:", e);
    }
  };

  const addManualPoint = () => {
    const next: SequencePoint = {
      id: `seq-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      x: 0,
      y: 0,
      clicks: 1,
    };
    update({ sequencePoints: [...points, next] });
  };

  const updatePoint = (index: number, patch: Partial<SequencePoint>) => {
    const next = [...points];
    const existing = next[index];
    if (!existing) return;
    next[index] = { ...existing, ...patch };
    update({ sequencePoints: next });
  };

  const removePoint = async (index: number) => {
    try {
      await invoke("remove_sequence_point", { index });
    } catch (e) {
      console.error("Failed to remove sequence point:", e);
    }
    const next = [...points];
    next.splice(index, 1);
    update({ sequencePoints: next });
  };

  const clearPoints = async () => {
    try {
      await invoke("clear_sequence_points");
    } catch (e) {
      console.error("Failed to clear sequence points:", e);
    }
    update({ sequencePoints: [] });
  };

  return (
    <div className="sectioncontainer">
      <div className="adv-card-header">
        <span className="adv-card-title">Sequence Clicking</span>
        <button
          type="button"
          className={`adv-toggle ${settings.sequenceEnabled ? "on" : "off"}`}
          onClick={toggleEnabled}
        >
          {settings.sequenceEnabled ? "On" : "Off"}
        </button>
      </div>

      <div className="seq-list">
        {points.length === 0 && (
          <div className="seq-empty">No sequence points configured.</div>
        )}
        {points.map((point, index) => (
          <div
            key={point.id}
            className={`seq-row ${activeSequenceIndex === index ? "active" : ""}`}
          >
            <span className="seq-index">{index + 1}</span>
            <div className="seq-fields">
              <input
                type="number"
                className="seq-input"
                value={point.x}
                onChange={(e) => updatePoint(index, { x: clampInt(e.target.value, -99999, 99999) })}
                title="X coordinate"
                min={-99999}
                max={99999}
              />
              <input
                type="number"
                className="seq-input"
                value={point.y}
                onChange={(e) => updatePoint(index, { y: clampInt(e.target.value, -99999, 99999) })}
                title="Y coordinate"
                min={-99999}
                max={99999}
              />
              <input
                type="number"
                className="seq-input seq-clicks"
                value={point.clicks}
                onChange={(e) =>
                  updatePoint(index, { clicks: clampInt(e.target.value, 1, 100000) })
                }
                title="Clicks"
                min={1}
                max={100000}
              />
            </div>
            <button
              type="button"
              className="seq-remove"
              onClick={() => removePoint(index)}
              title="Remove point"
            >
              ×
            </button>
          </div>
        ))}
      </div>

      <div className="seq-actions">
        <button type="button" className="seq-btn" onClick={addManualPoint}>
          Add Point
        </button>
        {picking ? (
          <button type="button" className="seq-btn seq-stop" onClick={stopPick}>
            Stop Picking
          </button>
        ) : (
          <button type="button" className="seq-btn seq-pick" onClick={startPick}>
            Pick from Screen
          </button>
        )}
        {points.length > 0 && (
          <button type="button" className="seq-btn seq-clear" onClick={clearPoints}>
            Clear
          </button>
        )}
      </div>

      {settings.sequenceEnabled && points.length === 0 && (
        <div className="seq-warning">Add at least one point to use sequence clicking.</div>
      )}
    </div>
  );
}
