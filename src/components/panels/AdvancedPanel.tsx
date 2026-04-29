import type { Settings } from "../../store";
import AdvancedPanelLayout from "./AdvancedPanelLayout";

interface Props {
  settings: Settings;
  update: (patch: Partial<Settings>) => void;
  on_pick_position: () => Promise<void>;
}

export default function AdvancedPanel({
  settings,
  update,
  on_pick_position,
}: Props) {
  return (
    <AdvancedPanelLayout
      settings={settings}
      update={update}
      on_pick_position={on_pick_position}
      compact={false}
      show_explanations
    />
  );
}
