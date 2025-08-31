import React, { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import CommandCompletion from './CommandCompletion';
import CommandHistory from './CommandHistory';
import SearchOverlay from './SearchOverlay';
import ScrollbackResults from './ScrollbackResults';
import { Terminal as XTerm } from 'xterm';
import { FitAddon } from 'xterm-addon-fit';
import { SearchAddon } from 'xterm-addon-search';
import 'xterm/css/xterm.css';

// HMR cleanup registry to avoid stale observers/listeners during hot reloads
const TERMINAL_HMR_CLEANUPS: Array<() => void> = [];
// Vite HMR dispose hook: cleanup all registered terminal resources before reload
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
if (import.meta && import.meta.hot) {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  import.meta.hot.dispose(() => {
    const fns = TERMINAL_HMR_CLEANUPS.splice(0);
    for (const fn of fns) {
      try { fn(); } catch {}
    }
  });
}

interface TerminalOutput {
  session_id: string;
  data: string;
}

interface TerminalProps {
  terminalId: string;
  isVisible?: boolean;
}

const Terminal: React.FC<TerminalProps> = ({ terminalId, isVisible }) => {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const termRef = useRef<XTerm | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const searchRef = useRef<SearchAddon | null>(null);
  const unlistenRef = useRef<Promise<() => void> | null>(null);
  const roRef = useRef<ResizeObserver | null>(null);
  const resizeTimerRef = useRef<number | undefined>(undefined);
  const [focused, setFocused] = useState(false);
  const [showCompletion, setShowCompletion] = useState(false);
  const [showHistory, setShowHistory] = useState(false);
  const [isAtPrompt, setIsAtPrompt] = useState(false);
  const [showResults, setShowResults] = useState(false);
  // Search UI state
  const [showSearch, setShowSearch] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const [caseSensitive, setCaseSensitive] = useState(false);
  const [useRegex, setUseRegex] = useState(false);
  const [wholeWord, setWholeWord] = useState(false);
  const [wrapSearch, setWrapSearch] = useState(true);
  const [scrollbackCount, setScrollbackCount] = useState(0);
  const [visibleCount, setVisibleCount] = useState(0);

  const cssVar = (name: string) => getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  const isTauri = () => {
    try { return typeof window !== 'undefined' && (('__TAURI__' in (window as any)) || ('__TAURI_INTERNALS__' in (window as any))); } catch { return false; }
  };
  const [ipcReady, setIpcReady] = useState(false);
  const [retryCount, setRetryCount] = useState(0);
  
  const safeInvoke = <T=any>(cmd: string, payload?: any): Promise<T> => {
    if (!isTauri()) return Promise.reject(new Error('Not running under Tauri'));
    return invoke<T>(cmd as any, payload).catch(error => {
      console.warn(`IPC call failed for ${cmd}:`, error);
      throw error;
    });
  };
  const safeListen = async <T=any>(event: string, handler: (e: any) => void): Promise<() => void> => {
    if (!isTauri()) return () => {};
    try {
      const un = await listen<T>(event as any, handler as any);
      return un;
    } catch (error) {
      console.warn(`Event listener setup failed for ${event}:`, error);
      return () => {};
    }
  };
  const toHex = (color: string) => {
    if (!color) return color;
    if (color.startsWith('#')) return color;
    const m = color.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/i);
    if (m) {
      const r = parseInt(m[1]).toString(16).padStart(2, '0');
      const g = parseInt(m[2]).toString(16).padStart(2, '0');
      const b = parseInt(m[3]).toString(16).padStart(2, '0');
      return `#${r}${g}${b}`;
    }
    return color;
  };

  const scheduleResize = (cols?: number, rows?: number) => {
    if (!isTauri()) return;
    if (resizeTimerRef.current) window.clearTimeout(resizeTimerRef.current);
    resizeTimerRef.current = window.setTimeout(() => {
      try {
        const t = termRef.current;
        const c = cols ?? t?.cols;
        const r = rows ?? t?.rows;
        if (t && c && r) {
          safeInvoke('resize_terminal', { terminalId, cols: c, rows: r }).catch(() => {});
        }
      } catch {}
    }, 120);
  };

  // Check IPC readiness on mount
  useEffect(() => {
    let mounted = true;
    let retries = 0;
    const maxRetries = 5;
    
    const checkIpcConnection = async () => {
      if (!mounted) return;
      
      try {
        // Try a simple backend call to test connectivity
        await safeInvoke('list_terminals'); // Use existing list_terminals command
        if (mounted) {
          setIpcReady(true);
          setRetryCount(0);
        }
      } catch (error) {
        console.warn(`IPC connection attempt ${retries + 1}/${maxRetries} failed:`, error);
        retries++;
        setRetryCount(retries);
        
        if (retries < maxRetries && mounted) {
          const backoffDelay = Math.min(1000 * Math.pow(2, retries), 10000);
          setTimeout(checkIpcConnection, backoffDelay);
        } else if (mounted) {
          console.error('Failed to establish IPC connection after maximum retries');
          // Continue anyway for offline development
          setIpcReady(false);
        }
      }
    };
    
    // Initial delay to allow backend to fully initialize
    if (isTauri()) {
      setTimeout(checkIpcConnection, 500);
    } else {
      setIpcReady(false); // Not in Tauri environment
    }
    
    return () => {
      mounted = false;
    };
  }, []);

  // Initialize xterm instance
  useEffect(() => {
    // Cleanup previous instance if terminalId changed
    if (termRef.current) {
      try { termRef.current.dispose(); } catch {}
      termRef.current = null;
    }
    if (unlistenRef.current) { unlistenRef.current.then(off => off()).catch(() => {}); unlistenRef.current = null; }

    const bg = cssVar('--bg') || '#1a1b26';
    const fg = cssVar('--fg') || '#c0caf5';
    const selection = cssVar('--selection') || '#33467c';

    const term = new XTerm({
      convertEol: true,
      scrollback: 10000,
      fontFamily: 'Fira Code, SF Mono, Monaco, Cascadia Code, Roboto Mono, Consolas, monospace',
      theme: {
        background: bg,
        foreground: fg,
        cursor: fg,
        cursorAccent: bg,
        selectionBackground: selection,
        selectionForeground: fg,
      },
    });
    const fit = new FitAddon();
    const search = new SearchAddon();
    term.loadAddon(fit);
    term.loadAddon(search);
    termRef.current = term;
    fitRef.current = fit;
    searchRef.current = search;

    if (containerRef.current) {
      term.open(containerRef.current);
      // Initial fit and resize backend
      setTimeout(() => {
        try {
          fit.fit();
          if (term.cols && term.rows) {
            scheduleResize(term.cols, term.rows);
          }
          term.focus();
        } catch {}
      }, 0);
    }

    // Wire input back to PTY
    const dataSub = term.onData((d) => {
      safeInvoke('write_to_terminal', { terminalId, data: d }).catch(() => {});
    });

    // Listen for backend output for this session
    const unlisten = safeListen<TerminalOutput>('terminal-output', (event) => {
      if (event.payload.session_id === terminalId) {
        term.write(event.payload.data);
        // Recompute visible count on new data for active query
        if (searchQuery.trim()) recomputeVisibleMatches();
      }
    });
    unlistenRef.current = unlisten as any;

    // Observe container resize
    if ('ResizeObserver' in window && containerRef.current) {
      roRef.current = new ResizeObserver(() => {
        try {
          fit.fit();
          scheduleResize();
        } catch {}
      });
      roRef.current.observe(containerRef.current);
    }

    const doCleanup = () => {
      try { dataSub.dispose(); } catch {}
      try { roRef.current?.disconnect(); roRef.current = null; } catch {}
      if (unlistenRef.current) { unlistenRef.current.then(off => off()).catch(() => {}); unlistenRef.current = null; }
      try { term.dispose(); } catch {}
      if (resizeTimerRef.current) { window.clearTimeout(resizeTimerRef.current); resizeTimerRef.current = undefined; }
    };
    TERMINAL_HMR_CLEANUPS.push(doCleanup);

    return () => {
      doCleanup();
      const i = TERMINAL_HMR_CLEANUPS.indexOf(doCleanup);
      if (i !== -1) TERMINAL_HMR_CLEANUPS.splice(i, 1);
    };
  }, [terminalId]);

  // Poll prompt status
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const atPrompt = await safeInvoke<boolean>('is_at_prompt', { terminalId }).catch(() => false);
        setIsAtPrompt(atPrompt);
      } catch {}
    }, 600);
    return () => clearInterval(interval);
  }, [terminalId]);

  // Keyboard shortcuts (global)
  useEffect(() => {
    if (!focused) return;
    const onKey = (event: KeyboardEvent) => {
      // Ctrl+Shift+F opens results modal
      if (event.ctrlKey && event.shiftKey && (event.key === 'F' || event.key === 'f')) {
        event.preventDefault();
        setShowResults(true);
        return;
      }
      if (event.ctrlKey && (event.key === 'f' || event.key === 'F')) {
        event.preventDefault();
        setShowSearch(true);
        return;
      }
      if (event.ctrlKey && (event.key === 'r' || event.key === 'R')) {
        event.preventDefault();
        if (isAtPrompt) setShowHistory(true);
        return;
      }
      if (event.ctrlKey && event.key === ' ') {
        event.preventDefault();
        if (isAtPrompt) setShowCompletion(true);
        return;
      }
      if (event.key === 'Escape') {
        if (showSearch) { setShowSearch(false); event.preventDefault(); return; }
        if (showResults) { setShowResults(false); event.preventDefault(); return; }
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [focused, isAtPrompt, showSearch, showResults]);

  const recomputeVisibleMatches = () => {
    const term = termRef.current as any;
    if (!term || !searchQuery.trim()) { setVisibleCount(0); return; }
    try {
      const buf = term.buffer.active;
      const top = buf.viewportY;
      const rows = term.rows;
      let count = 0;
      let re: RegExp | null = null;
      if (useRegex || wholeWord) {
        try {
          const pattern = useRegex ? searchQuery : `\\b${searchQuery.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}\\b`;
          re = new RegExp(pattern, caseSensitive ? 'g' : 'gi');
        } catch { re = null; }
      }
      for (let y = top; y < top + rows; y++) {
        const line = buf.getLine(y);
        const text = line ? line.translateToString(true) : '';
        if (!text) continue;
        if (re) {
          let m; re.lastIndex = 0;
          while ((m = re.exec(text)) !== null) { count += 1; if (m[0]?.length === 0) re.lastIndex++; }
        } else {
          const needle = caseSensitive ? searchQuery : searchQuery.toLowerCase();
          const hay = caseSensitive ? text : text.toLowerCase();
          let idx = 0;
          while (needle && (idx = hay.indexOf(needle, idx)) !== -1) { count += 1; idx += Math.max(needle.length, 1); }
        }
      }
      setVisibleCount(count);
    } catch { setVisibleCount(0); }
  };

  // Fit when becoming visible (tab switch)
  useEffect(() => {
    if (isVisible && termRef.current && fitRef.current) {
      setTimeout(() => {
        try {
          fitRef.current!.fit();
          const t = termRef.current!;
          if (t.cols && t.rows) {
            scheduleResize(t.cols, t.rows);
          }
          t.focus();
        } catch {}
      }, 0);
    }
  }, [isVisible, terminalId]);

  // Update scrollback count and highlight when query changes
  useEffect(() => {
    const search = searchRef.current;
    if (!search || !termRef.current) { setVisibleCount(0); setScrollbackCount(0); return; }
    if (!searchQuery.trim()) { search.clearDecorations(); setVisibleCount(0); setScrollbackCount(0); return; }

    const accent = toHex(cssVar('--accent') || '#7aa2f7');
    const accent2 = toHex(cssVar('--accent-2') || '#10b981');

    // Highlight all using decorations; moves to first match
    search.findNext(searchQuery, {
      regex: useRegex,
      caseSensitive,
      wholeWord,
      incremental: false,
      decorations: {
        matchBackground: accent,
        activeMatchBackground: accent2,
        matchOverviewRuler: accent,
        activeMatchColorOverviewRuler: accent2,
      }
    });

    // Use backend for total scrollback count
    safeInvoke('search_scrollback', {
      terminalId,
      query: searchQuery,
      caseSensitive,
      useRegex,
      limit: 2000,
    }).then((matches: any) => setScrollbackCount(Array.isArray(matches) ? matches.length : 0))
      .catch(() => setScrollbackCount(0));
  }, [searchQuery, caseSensitive, useRegex, wholeWord, terminalId]);

  const gotoNext = () => {
    const search = searchRef.current; if (!search) return;
    const accent = toHex(cssVar('--accent') || '#7aa2f7');
    const accent2 = toHex(cssVar('--accent-2') || '#10b981');
    search.findNext(searchQuery, { regex: useRegex, caseSensitive, wholeWord, incremental: false, decorations: { matchBackground: accent, activeMatchBackground: accent2, matchOverviewRuler: accent, activeMatchColorOverviewRuler: accent2 } });
  };
  const gotoPrev = () => {
    const search = searchRef.current; if (!search) return;
    const accent = toHex(cssVar('--accent') || '#7aa2f7');
    const accent2 = toHex(cssVar('--accent-2') || '#10b981');
    search.findPrevious(searchQuery, { regex: useRegex, caseSensitive, wholeWord, incremental: false, decorations: { matchBackground: accent, activeMatchBackground: accent2, matchOverviewRuler: accent, activeMatchColorOverviewRuler: accent2 } });
  };

  // Listen for decoration result changes (visible matches)
  useEffect(() => {
    const s = searchRef.current;
    if (!s) return;
    const d = s.onDidChangeResults?.(({ resultCount }) => {
      setVisibleCount(typeof resultCount === 'number' ? resultCount : 0);
    });
    return () => { if (d) (d as any).dispose?.(); };
  }, []);

  const handleJumpToMatch = (lineIndex: number) => {
    const term = termRef.current as any;
    if (!term) return;
    // Try xterm scrollToLine if available
    if (typeof term.scrollToLine === 'function') {
      term.scrollToLine(lineIndex);
    } else {
      // Fallback: compute delta from top
      const currentTop = term.buffer.active.viewportY;
      const delta = lineIndex - currentTop;
      term.scrollLines(delta);
    }
    // Kick search to highlight at/near the match
    const search = searchRef.current;
    if (search && searchQuery.trim()) {
      gotoNext();
    }
    setShowResults(false);
  };

  return (
    <div className="terminal-container">
      {!ipcReady && isTauri() && retryCount > 0 && (
        <div className="ipc-status" style={{ 
          position: 'absolute', 
          top: '8px', 
          right: '8px', 
          background: 'rgba(255, 165, 0, 0.9)', 
          color: 'white', 
          padding: '4px 8px', 
          borderRadius: '4px', 
          fontSize: '12px',
          zIndex: 1000
        }}>
          Connecting to backend... (attempt {retryCount})
        </div>
      )}
      <div
        ref={containerRef}
        className="terminal-grid"
        onClick={() => { termRef.current?.focus(); }}
        onFocus={() => setFocused(true)}
        onBlur={() => setFocused(false)}
        tabIndex={0}
      />

      {/* Shell integration UI indicator */}
      {isAtPrompt && (
        <div className="shell-status">
          <span className="prompt-indicator">●</span>
          <span className="status-text">Ready for commands</span>
          <span className="keybind-hint">Ctrl+R: History • Ctrl+Space: Complete</span>
        </div>
      )}

      {/* Scrollback results modal */}
      <ScrollbackResults
        visible={showResults}
        terminalId={terminalId}
        query={searchQuery}
        caseSensitive={caseSensitive}
        useRegex={useRegex}
        wholeWord={wholeWord}
        onClose={() => setShowResults(false)}
        onJump={handleJumpToMatch}
      />

      {/* Search overlay */}
      <SearchOverlay
        visible={showSearch}
        query={searchQuery}
        onChangeQuery={setSearchQuery}
        onClose={() => setShowSearch(false)}
        count={visibleCount}
        totalCount={scrollbackCount}
        index={0}
        onNext={gotoNext}
        onPrev={gotoPrev}
        caseSensitive={caseSensitive}
        onToggleCaseSensitive={() => setCaseSensitive((v) => !v)}
        useRegex={useRegex}
        onToggleRegex={() => setUseRegex((v) => !v)}
        wholeWord={wholeWord}
        onToggleWholeWord={() => setWholeWord((v) => !v)}
        wrapSearch={wrapSearch}
        onToggleWrap={() => setWrapSearch((v) => !v)}
        onOpenResults={() => setShowResults(true)}
      />

      {/* Command completion overlay */}
      <CommandCompletion
        terminalId={terminalId}
        currentInput={''}
        cursorPosition={0}
        onSuggestionSelect={(s) => safeInvoke('write_to_terminal', { terminalId, data: s }).catch(() => {})}
        visible={showCompletion}
        onClose={() => setShowCompletion(false)}
      />

      {/* Command history overlay */}
      <CommandHistory
        terminalId={terminalId}
        visible={showHistory}
        onClose={() => setShowHistory(false)}
        onCommandSelect={(cmd) => safeInvoke('write_to_terminal', { terminalId, data: cmd + '\r' }).catch(() => {})}
      />
    </div>
  );
};

export default Terminal;
