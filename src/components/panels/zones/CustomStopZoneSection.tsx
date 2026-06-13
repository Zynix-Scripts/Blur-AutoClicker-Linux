import { useCallback, useEffect, useRef, useState } from 'react';
import type { Settings } from '../../../store';
import { useTranslation } from '../../../i18n';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import {
  Disableable,
  NumInput,
  ToggleBtn,
  CardDivider,
  InfoIcon,
} from '../advanced/shared';

interface Props {
  settings: Settings;
  update: (patch: Partial<Settings>) => void;
  showInfo: boolean;
}

interface CustomStopZonePickedPayload {
  x: number;
  y: number;
  width: number;
  height: number;
}

export default function CustomStopZoneSection({
  settings,
  update,
  showInfo,
}: Props) {
  const { t } = useTranslation();
  const [drawingZone, setDrawingZone] = useState(false);
  const updateRef = useRef(update);

  useEffect(() => {
    updateRef.current = update;
  }, [update]);

  useEffect(() => {
    let disposed = false;
    let unlistenPicked: (() => void) | null = null;
    let unlistenEnded: (() => void) | null = null;

    void listen<CustomStopZonePickedPayload>(
      'custom-stop-zone-picked',
      (event) => {
        updateRef.current({
          customStopZoneEnabled: true,
          customStopZoneX: event.payload.x,
          customStopZoneY: event.payload.y,
          customStopZoneWidth: Math.max(1, event.payload.width),
          customStopZoneHeight: Math.max(1, event.payload.height),
        });
        setDrawingZone(false);
      },
    ).then((cleanup) => {
      if (disposed) {
        cleanup();
      } else {
        unlistenPicked = cleanup;
      }
    });

    void listen('custom-stop-zone-pick-ended', () => {
      setDrawingZone(false);
    }).then((cleanup) => {
      if (disposed) {
        cleanup();
      } else {
        unlistenEnded = cleanup;
      }
    });

    return () => {
      disposed = true;
      unlistenPicked?.();
      unlistenEnded?.();
      void invoke('cancel_custom_stop_zone_pick');
    };
  }, []);

  const startCustomStopZonePick = useCallback(async () => {
    setDrawingZone(true);
    try {
      await invoke('start_custom_stop_zone_pick');
    } catch (error) {
      setDrawingZone(false);
      console.error('Failed to start custom stop zone picker', error);
    }
  }, []);

  const cancelCustomStopZonePick = useCallback(async () => {
    setDrawingZone(false);
    try {
      await invoke('cancel_custom_stop_zone_pick');
    } catch (error) {
      console.error('Failed to cancel custom stop zone picker', error);
    }
  }, []);

  useEffect(() => {
    if (!drawingZone) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') {
        return;
      }

      event.preventDefault();
      event.stopPropagation();
      void cancelCustomStopZonePick();
    };

    window.addEventListener('keydown', handleKeyDown, true);
    return () => window.removeEventListener('keydown', handleKeyDown, true);
  }, [cancelCustomStopZonePick, drawingZone]);

  return (
    <div className="adv-sectioncontainer">
      <div className="adv-card-header">
        <div
          style={{
            display: 'inline-flex',
            alignItems: 'center',
            gap: '0.5rem',
          }}
        >
          {showInfo ? (
            <InfoIcon text={t('advanced.customStopZoneDescription')} />
          ) : null}
          <span className="adv-card-title">{t('advanced.customStopZone')}</span>
        </div>
        <ToggleBtn
          value={settings.customStopZoneEnabled}
          onChange={(v) => {
            if (!v && drawingZone) {
              void cancelCustomStopZonePick();
            }
            update({ customStopZoneEnabled: v });
          }}
        />
      </div>
      <CardDivider />
      <Disableable enabled={settings.customStopZoneEnabled}>
        <div className="adv-stop-zone-body">
          <div className="adv-stop-zone-controls">
            <div className="adv-stop-zone-grid">
              <label
                className="adv-numbox-sm adv-sequence-coord adv-stop-zone-input"
                style={{ gap: '6px' }}
              >
                <span
                  className="adv-unit"
                  style={{ minWidth: '0.75rem', textAlign: 'center' }}
                >
                  X
                </span>
                <NumInput
                  value={settings.customStopZoneX}
                  onChange={(v) => update({ customStopZoneX: v })}
                  style={{ flex: 1, width: '100%', textAlign: 'left' }}
                />
              </label>
              <label
                className="adv-numbox-sm adv-sequence-coord adv-stop-zone-input"
                style={{ gap: '6px' }}
              >
                <span
                  className="adv-unit"
                  style={{ minWidth: '0.75rem', textAlign: 'center' }}
                >
                  Y
                </span>
                <NumInput
                  value={settings.customStopZoneY}
                  onChange={(v) => update({ customStopZoneY: v })}
                  style={{ flex: 1, width: '100%', textAlign: 'left' }}
                />
              </label>
              <label
                className="adv-numbox-sm adv-sequence-coord adv-stop-zone-input"
                style={{ gap: '6px' }}
              >
                <span
                  className="adv-unit"
                  style={{ minWidth: '0.75rem', textAlign: 'center' }}
                >
                  W
                </span>
                <NumInput
                  value={settings.customStopZoneWidth}
                  onChange={(v) => update({ customStopZoneWidth: v })}
                  min={1}
                  style={{ flex: 1, width: '100%', textAlign: 'left' }}
                />
              </label>
              <label
                className="adv-numbox-sm adv-sequence-coord adv-stop-zone-input"
                style={{ gap: '6px' }}
              >
                <span
                  className="adv-unit"
                  style={{ minWidth: '0.75rem', textAlign: 'center' }}
                >
                  H
                </span>
                <NumInput
                  value={settings.customStopZoneHeight}
                  onChange={(v) => update({ customStopZoneHeight: v })}
                  min={1}
                  style={{ flex: 1, width: '100%', textAlign: 'left' }}
                />
              </label>
            </div>
            <div className="adv-sequence-actions adv-stop-zone-actions">
              <button
                type="button"
                className="adv-secondary-btn"
                onClick={() => {
                  void (drawingZone
                    ? cancelCustomStopZonePick()
                    : startCustomStopZonePick());
                }}
              >
                {drawingZone
                  ? t('advanced.customStopZoneCancelDrawing')
                  : t('advanced.customStopZoneDraw')}
              </button>
            </div>
          </div>
        </div>
      </Disableable>
    </div>
  );
}
