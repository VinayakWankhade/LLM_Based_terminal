import React, { useEffect, useRef } from 'react';

interface SearchOverlayProps {
  visible: boolean;
  query: string;
  onChangeQuery: (q: string) => void;
  onClose: () => void;
  count: number; // visible screen matches
  totalCount?: number; // overall matches including scrollback
  index: number; // 1-based active index, 0 if none
  onNext: () => void;
  onPrev: () => void;
  caseSensitive: boolean;
  onToggleCaseSensitive: () => void;
  useRegex: boolean;
  onToggleRegex: () => void;
  wholeWord: boolean;
  onToggleWholeWord: () => void;
  wrapSearch: boolean;
  onToggleWrap: () => void;
  onOpenResults: () => void;
}

const SearchOverlay: React.FC<SearchOverlayProps> = ({
  visible,
  query,
  onChangeQuery,
  onClose,
  count,
  totalCount,
  index,
  onNext,
  onPrev,
  caseSensitive,
  onToggleCaseSensitive,
  useRegex,
  onToggleRegex,
  wholeWord,
  onToggleWholeWord,
  wrapSearch,
  onToggleWrap,
  onOpenResults,
}) => {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (visible) {
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [visible]);

  if (!visible) return null;

  return (
    <div className="search-overlay" role="dialog" aria-label="Search">
      <input
        ref={inputRef}
        className="search-input"
        type="text"
        placeholder="Find... (Enter/Shift+Enter to navigate)"
        value={query}
        onChange={(e) => onChangeQuery(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter') {
            e.preventDefault();
            e.stopPropagation();
            if (e.shiftKey) onPrev(); else onNext();
          } else if (e.key === 'Escape') {
            e.preventDefault();
            e.stopPropagation();
            onClose();
          }
        }}
      />
      <div className="search-controls">
        <button className="terminal-control-btn" onClick={onPrev} title="Previous (Shift+Enter)">▲</button>
        <button className="terminal-control-btn" onClick={onNext} title="Next (Enter)">▼</button>
        <span className="search-count">{index > 0 ? index : 0}/{count}{typeof totalCount === 'number' && totalCount !== count ? ` • ${totalCount} total` : ''}</span>
        <label className="search-toggle">
          <input type="checkbox" checked={caseSensitive} onChange={onToggleCaseSensitive} /> Aa
        </label>
        <label className="search-toggle">
          <input type="checkbox" checked={useRegex} onChange={onToggleRegex} /> .* 
        </label>
        <label className="search-toggle">
          <input type="checkbox" checked={wholeWord} onChange={onToggleWholeWord} /> \b
        </label>
        <label className="search-toggle">
          <input type="checkbox" checked={wrapSearch} onChange={onToggleWrap} /> wrap
        </label>
        <button className="terminal-control-btn" onClick={onOpenResults} title="Open results (Ctrl+Shift+F)">☰</button>
        <button className="terminal-control-btn" onClick={onClose} title="Close (Esc)">×</button>
      </div>
    </div>
  );
};

export default SearchOverlay;
