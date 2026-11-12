import { useState } from "react";
import Diagnostics from "./Diagnostics";
import StudySessions from "./StudySessions";
import "./App.css";

export default function App() {
  const [diagnostics, setDiagnostics] = useState(false);
  return (
    <main className="workspace">
      <header className="workspace-header">
        <div>
          <p className="eyebrow">Your study companion</p>
          <h1>Mission Control</h1>
          <p className="subtitle">Focus on consistency. Your progress stays yours.</p>
        </div>
        <button
          type="button"
          className="secondary-button diagnostics-toggle"
          onClick={() => setDiagnostics((previous) => !previous)}
          aria-pressed={diagnostics}
        >
          {diagnostics ? "Back to study" : "Diagnostics"}
        </button>
      </header>
      {diagnostics ? <Diagnostics /> : <StudySessions />}
    </main>
  );
}
