import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface ScrollMatch {
  line_index: number;
  start: number;
  end: number;
  line: string;
}

interface ContextLine {
  line_index: number;
  line: string;
}

interface Props {
  visible: boolean;
  terminalId: string;
  query: string;
  caseSensitive: boolean;
  useRegex: boolean;
  wholeWord: boolean; // hint for frontend rendering only; backend doesn't need it when useRegex=true
  onClose: () => void;
  onJump?: (lineIndex: number, matchText: string) => void;
}

const highlight = (line: string, start: number, end: number) => {
  return (
    <>
      <span>{line.slice(0, start)}</span>
      <span className="search-highlight-active">{line.slice(start, end) || ' '}</span>
      <span>{line.slice(end)}</span>
    </>
  );
};

const ScrollbackResults: React.FC<Props> = ({ visible, terminalId, query, caseSensitive, useRegex, wholeWord: _wholeWord, onClose, onJump }) => {
  const [results, setResults] = useState<ScrollMatch[]>([]);
  const [selected, setSelected] = useState<number>(0);
  const [context, setContext] = useState<ContextLine[]>([]);
  const [loading, setLoading] = useState(false);
  const [pathsOnly, setPathsOnly] = useState(false);

  // Keyboard navigation inside the results modal
  // Derived filtered results
  const filteredResults = React.useMemo(() => {
    if (!pathsOnly) return results;
    const pathLike = /(?:[A-Za-z]:\\|\\\\|\/)[^\s]+/;
    return results.filter((r) => pathLike.test(r.line));
  }, [results, pathsOnly]);

  useEffect(() => {
    if (!visible) return;
    const onKey = (e: KeyboardEvent) => {
      if (!visible) return;
      if (e.key === 'Escape') { e.preventDefault(); onClose(); }
      if (e.key === 'ArrowDown') { e.preventDefault(); setSelected((s) => Math.min(s + 1, Math.max(0, results.length - 1))); }
      if (e.key === 'ArrowUp') { e.preventDefault(); setSelected((s) => Math.max(s - 1, 0)); }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [visible, results.length, onClose]);

  useEffect(() => {
    if (!visible || !query.trim()) {
      setResults([]);
      setSelected(0);
      setContext([]);
      return;
    }
    setLoading(true);
    invoke<ScrollMatch[]>('search_scrollback', {
      terminalId,
      query,
      caseSensitive,
      useRegex,
      limit: 2000,
    })
      .then((res) => setResults(res))
      .catch(() => setResults([]))
      .finally(() => setLoading(false));
  }, [visible, terminalId, query, caseSensitive, useRegex]);

  useEffect(() => {
    if (!visible) return;
    if (results.length === 0) { setContext([]); return; }
    const sel = Math.max(0, Math.min(selected, results.length - 1));
    const lineIndex = results[sel].line_index;
    invoke<ContextLine[]>('get_scrollback_context', {
      terminalId,
      lineIndex,
      before: 4,
      after: 6,
    }).then(setContext).catch(() => setContext([]));
  }, [visible, terminalId, filteredResults, selected]);

  if (!visible) return null;

  // Copy helpers
  const copySelectedLine = async () => {
    if (!filteredResults.length) return;
    const sel = Math.max(0, Math.min(selected, filteredResults.length - 1));
    if (navigator.clipboard && 'writeText' in navigator.clipboard) {
      await navigator.clipboard.writeText(filteredResults[sel].line);
    }
  };
  const copySelectedMatch = async () => {
    if (!filteredResults.length) return;
    const sel = Math.max(0, Math.min(selected, filteredResults.length - 1));
    const m = filteredResults[sel];
    if (navigator.clipboard && 'writeText' in navigator.clipboard) {
      await navigator.clipboard.writeText(m.line.slice(m.start, m.end));
    }
  };
  const copyContext = async () => {
    if (!context.length) return;
    if (navigator.clipboard && 'writeText' in navigator.clipboard) {
      await navigator.clipboard.writeText(context.map((c) => c.line).join('\n'));
    }
  };

  return (
    <div className="search-results-overlay">
      <div className="search-results-modal">
            <div className="results-header">
              <div className="results-title">Search results</div>
              <div className="results-actions">
                <span className="results-count">{loading ? 'Searching...' : `${filteredResults.length} matches`}</span>
                <label className="search-toggle"><input type="checkbox" checked={pathsOnly} onChange={() => setPathsOnly((v) => !v)} /> paths</label>
                <button className="terminal-control-btn" title="Copy line" onClick={copySelectedLine}>⧉ line</button>
                <button className="terminal-control-btn" title="Copy match" onClick={copySelectedMatch}>⧉ match</button>
                <button className="terminal-control-btn" title="Copy context" onClick={copyContext}>⧉ ctx</button>
                <button className="terminal-control-btn" title="Jump to match" onClick={() => {
                  const m = filteredResults[Math.max(0, Math.min(selected, filteredResults.length - 1))];
                  if (m && onJump) onJump(m.line_index, m.line.slice(m.start, m.end));
                }}>↧ jump</button>
                <button className="close-button" onClick={onClose}>×</button>
              </div>
            </div>
        <div className="results-content">
          <div className="results-list">
            {filteredResults.length === 0 && (
              <div className="history-empty">No matches</div>
            )}
            {filteredResults.map((m, idx) => (
              <div
                key={`${m.line_index}-${m.start}-${m.end}-${idx}`}
                className={`results-item ${idx === selected ? 'selected' : ''}`}
                onClick={() => setSelected(idx)}
                onDoubleClick={() => onJump?.(m.line_index, m.line.slice(m.start, m.end))}
              >
                <div className="results-line-index">{m.line_index}</div>
                <div className="results-line-text">{highlight(m.line, m.start, m.end)}</div>
              </div>
            ))}
          </div>
          <div className="results-preview">
            <pre className="results-pre">
              {context.map((c) => {
                const active = filteredResults[selected] && c.line_index === filteredResults[selected].line_index;
                let content: React.ReactNode = c.line;
                if (active) {
                  const m = filteredResults[selected];
                  content = (
                    <>
                      {c.line.slice(0, m.start)}
                      <span className="search-highlight-active">{c.line.slice(m.start, m.end) || ' '}</span>
                      {c.line.slice(m.end)}
                    </>
                  );
                }
                return (
                  <div key={c.line_index} className={`results-pre-line ${active ? 'active' : ''}`}>
                    <span className="results-pre-idx">{String(c.line_index).padStart(5, ' ')}</span>
                    <span className="results-pre-text">{content}</span>
                  </div>
                );
              })}
            </pre>
          </div>
        </div>
        <div className="results-footer">
          <span className="completion-hint">Enter: select • Esc: close • ↑/↓: navigate</span>
        </div>
      </div>
    </div>
  );
};

export default ScrollbackResults;
