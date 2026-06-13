import type { Settings } from "../../store";
import AdvancedPanelLayout from "./AdvancedPanelLayout";

interface Props {
  settings: Settings;
  update: (patch: Partial<Settings>) => void;
  on_pick_position: () => Promise<void>;
  activeSequenceIndex: number | null;
}

export default function AdvancedPanelCompact({
  settings,
  update,
  on_pick_position,
  activeSequenceIndex,
}: Props) {
  return (
    <AdvancedPanelLayout
      settings={settings}
      update={update}
      on_pick_position={on_pick_position}
      compact
      show_explanations={false}
      activeSequenceIndex={activeSequenceIndex}
    />
  );
}
