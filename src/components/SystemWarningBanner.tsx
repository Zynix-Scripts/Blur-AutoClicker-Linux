import { useState } from "react";
import "./SystemWarningBanner.css";

interface SystemWarningBannerProps {
  message: string;
  on_dismiss: () => void;
}

export default function SystemWarningBanner({
  message,
  on_dismiss,
}: SystemWarningBannerProps) {
  const [expanded, set_expanded] = useState(false);

  const lines = message.split("\n");
  const summary = lines[0];
  const details = lines.slice(1).join("\n");

  return (
    <div className="sys-warn-banner">
      <span className="sys-warn-icon">⚠</span>
      <div className="sys-warn-body">
        <span className="sys-warn-summary">{summary}</span>
        {details && (
          <>
            {expanded && (
              <pre className="sys-warn-details">{details}</pre>
            )}
            <button
              className="sys-warn-toggle"
              onClick={() => set_expanded((v) => !v)}
            >
              {expanded ? "less" : "how to fix"}
            </button>
          </>
        )}
      </div>
      <button className="sys-warn-dismiss" onClick={on_dismiss} title="Dismiss">
        ✕
      </button>
    </div>
  );
}
