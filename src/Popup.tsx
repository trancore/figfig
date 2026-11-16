import { useEffect, useRef, type ReactNode } from "react";
import type { CompletionResponse } from "./types";

const icons: Record<string, ReactNode> = {
  command: <path d="m5 6 5 5-5 5m8 1h6" />,
  subcommand: <path d="M6 4v8a3 3 0 0 0 3 3h10m-4-4 4 4-4 4" />,
  option: <path d="M5 8h14M5 16h14M9 5v6m6 2v6" />,
  branch: <><circle cx="6" cy="5" r="2" /><circle cx="18" cy="6" r="2" /><circle cx="6" cy="19" r="2" /><path d="M6 7v10m12-9a7 7 0 0 1-7 7H6" /></>,
  script: <path d="m8 5 11 7-11 7Z" />,
  directory: <path d="M3 7a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z" />,
  file: <path d="M14 3H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9Zm0 0v6h6" />,
};

export function Popup({ completion }: { completion: CompletionResponse }) {
  const selectedRow = useRef<HTMLLIElement>(null);

  useEffect(() => {
    selectedRow.current?.scrollIntoView?.({ block: "nearest" });
  }, [completion.selected]);

  return (
    <section className="completion-panel" aria-label="figfig 補完候補">
      <header className="completion-header">
        <span className="wordmark">fig<span>fig</span></span>
        <span className="completion-tag">SUGGESTIONS</span>
      </header>
      <ul
        role="listbox"
        aria-label="コマンド候補"
        aria-activedescendant={`candidate-${completion.selected}`}
      >
        {completion.candidates.map((candidate, index) => {
          const isSelected = index === completion.selected;
          return (
            <li
              key={`${candidate.kind}:${candidate.label}`}
              id={`candidate-${index}`}
              role="option"
              aria-selected={isSelected}
              ref={isSelected ? selectedRow : undefined}
            >
              <span className={`candidate-icon ${candidate.kind}`} aria-hidden="true">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.6" strokeLinecap="round" strokeLinejoin="round" focusable="false">
                  {icons[candidate.kind] ?? icons.file}
                </svg>
              </span>
              <span className="candidate-copy">
                <span className="candidate-label">{candidate.label}</span>
                <span className="candidate-description">{candidate.description}</span>
              </span>
              <span className="candidate-action" aria-hidden="true">
                {isSelected && <kbd className="accept-hint">↵</kbd>}
              </span>
            </li>
          );
        })}
      </ul>
      <footer className="completion-footer">
        <span><kbd>↑</kbd><kbd>↓</kbd> 選択</span>
        <span><kbd>tab</kbd> 挿入</span>
        <span><kbd>esc</kbd> 閉じる</span>
        <span className="local-dot">LOCAL</span>
      </footer>
    </section>
  );
}
