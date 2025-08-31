import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import PaneLayout, { PaneNode, SplitDirection } from './components/PaneLayout';
import AIPanel from './components/AIPanel';
import WorkflowsPanel from './components/WorkflowsPanel';
import SettingsPanel from './components/SettingsPanel';
import AdvancedFeatures from './components/AdvancedFeatures';

function genId(prefix = 'pane') {
  return `${prefix}-${Math.random().toString(36).slice(2, 8)}-${Date.now().toString(36)}`;
}

interface TerminalOutput {
  session_id: string;
  data: string;
}

interface TerminalTab {
  id: string; // tab id
  title: string;
  active: boolean;
  root: PaneNode; // layout tree
  activePaneId: string; // currently focused leaf pane id
}

// Helpers to traverse/modify pane trees
function findNode(node: PaneNode, id: string): PaneNode | null {
  if (node.id === id) return node;
  if (node.type === 'split') {
    for (const child of node.children) {
      const found = findNode(child, id);
      if (found) return found;
    }
  }
  return null;
}

function replaceNode(node: PaneNode, id: string, newNode: PaneNode): PaneNode {
  if (node.id === id) return newNode;
  if (node.type === 'split') {
    return {
      ...node,
      children: node.children.map((c) => replaceNode(c, id, newNode)),
    };
  }
  return node;
}

function removeNode(node: PaneNode, id: string): PaneNode | null {
  if (node.id === id) return null;
  if (node.type === 'split') {
    const left = removeNode(node.children[0], id);
    const right = removeNode(node.children[1], id);
    if (left && right) {
      return { ...node, children: [left, right] };
    }
    return left ?? right; // collapse single-child splits
  }
  return node;
}

function App() {
  const [tabs, setTabs] = useState<TerminalTab[]>([]);
  const [showAIPanel, setShowAIPanel] = useState(false);
  const [showWorkflows, setShowWorkflows] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [settings, setSettings] = useState<any | null>(null);

  useEffect(() => {
    // Listen for terminal output from the backend (kept for potential global behaviours)
    const unlisten = listen<TerminalOutput>('terminal-output', () => {});
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Load settings once and apply theme + font size
  useEffect(() => {
    (async () => {
      try {
        const s = await invoke<any>('get_settings');
        setSettings(s);
        applyAppearance(s);
      } catch {}
    })();
  }, []);

  const applyAppearance = (s: any) => {
    const theme = s?.theme || 'dark';
    const font = s?.font_size || 14;
    document.documentElement.setAttribute('data-theme', theme);
    document.documentElement.style.setProperty('--font-size', `${font}px`);
  };

  const parseKey = (def: string) => {
    const parts = def.toLowerCase().split('+').map(p => p.trim()).filter(Boolean);
    const mods = { ctrl: false, shift: false, alt: false };
    let key: string | null = null;
    for (const p of parts) {
      if (p === 'ctrl' || p === 'control') mods.ctrl = true;
      else if (p === 'shift') mods.shift = true;
      else if (p === 'alt' || p === 'option') mods.alt = true;
      else key = p;
    }
    return { mods, key };
  };

  const matchKey = (e: KeyboardEvent, binding: string) => {
    const { mods, key } = parseKey(binding);
    if (!key) return false;
    const pressed = (k: string) => (e.key.toLowerCase() === k || e.code.toLowerCase().endsWith(k));
    return (!!mods.ctrl === e.ctrlKey) && (!!mods.shift === e.shiftKey) && (!!mods.alt === e.altKey) && pressed(key);
  };

  // Global shortcuts from settings
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const kb = settings?.keybindings;
      if (kb) {
        if (matchKey(e, kb.open_workflows)) { e.preventDefault(); setShowWorkflows(true); return; }
        if (matchKey(e, kb.open_ai_panel)) { e.preventDefault(); setShowAIPanel((v) => !v); return; }
        if (matchKey(e, kb.split_vertical)) { e.preventDefault(); splitActivePane('vertical'); return; }
        if (matchKey(e, kb.split_horizontal)) { e.preventDefault(); splitActivePane('horizontal'); return; }
        if (matchKey(e, kb.close_pane)) { e.preventDefault(); closeActivePane(); return; }
        // Ctrl+Shift+A for advanced features
        if (e.ctrlKey && e.shiftKey && (e.key === 'A' || e.key === 'a')) { e.preventDefault(); setShowAdvanced(true); return; }
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [settings]);

  const createTabWithTerminal = async () => {
    try {
      const terminalId = await invoke<string>('create_terminal', {
        cols: 80,
        rows: 24,
        shell: null,
        workingDir: null,
      });
      const paneId = genId();
      const root: PaneNode = { id: paneId, type: 'leaf', terminalId };
      const newTab: TerminalTab = {
        id: genId('tab'),
        title: `Tab ${tabs.length + 1}`,
        active: true,
        root,
        activePaneId: paneId,
      };
      setTabs((prev) => [
        ...prev.map((t) => ({ ...t, active: false })),
        newTab,
      ]);
    } catch (error) {
      console.error('Failed to create initial terminal for tab:', error);
    }
  };

  const closeTab = async (tabId: string) => {
    const tab = tabs.find((t) => t.id === tabId);
    if (!tab) return;

    // Collect terminal IDs to close
    const collect = (node: PaneNode, acc: string[]) => {
      if (node.type === 'leaf') acc.push(node.terminalId);
      else node.children.forEach((c) => collect(c, acc));
    };
    const ids: string[] = [];
    collect(tab.root, ids);
    try {
      await Promise.all(ids.map((terminalId) => invoke('close_terminal', { terminalId })));
    } catch (e) {
      console.warn('Some terminals failed to close:', e);
    }
    setTabs((prev) => {
      const filtered = prev.filter((t) => t.id !== tabId);
      if (filtered.length > 0 && !filtered.some((t) => t.active)) {
        filtered[0].active = true;
      }
      return filtered;
    });
  };

  const switchTab = (tabId: string) => {
    setTabs((prev) => prev.map((t) => ({ ...t, active: t.id === tabId })));
  };

  const activeTab = tabs.find((t) => t.active) || null;

  const focusPane = (paneId: string) => {
    if (!activeTab) return;
    setTabs((prev) => prev.map((t) => (t.id === activeTab.id ? { ...t, activePaneId: paneId } : t)));
  };

  const splitActivePane = async (direction: SplitDirection) => {
    if (!activeTab) return;
    const target = findNode(activeTab.root, activeTab.activePaneId);
    if (!target || target.type !== 'leaf') return;

    try {
      const terminalId = await invoke<string>('create_terminal', {
        cols: 80,
        rows: 24,
        shell: null,
        workingDir: null,
      });
      const newLeaf: PaneNode = { id: genId(), type: 'leaf', terminalId };
      const splitNode: PaneNode = {
        id: genId('split'),
        type: 'split',
        direction,
        children: [target, newLeaf],
        sizes: [1, 1],
      };

      setTabs((prev) => prev.map((t) => {
        if (t.id !== activeTab.id) return t;
        const newRoot = replaceNode(t.root, target.id, splitNode);
        return { ...t, root: newRoot, activePaneId: newLeaf.id };
      }));
    } catch (error) {
      console.error('Failed to split pane:', error);
    }
  };

  const closeActivePane = async () => {
    if (!activeTab) return;
    const target = findNode(activeTab.root, activeTab.activePaneId);
    if (!target || target.type !== 'leaf') return;

    try {
      await invoke('close_terminal', { terminalId: target.terminalId });
    } catch (e) {
      console.warn('Failed to close terminal:', e);
    }

    setTabs((prev) => prev.map((t) => {
      if (t.id !== activeTab.id) return t;
      const newRoot = removeNode(t.root, target.id);
      if (!newRoot) {
        // Tab became empty: create a fresh terminal synchronously-like
        // We cannot await here; instead, trigger async creation and keep tab as-is for now
        (async () => {
          const terminalId = await invoke<string>('create_terminal', { cols: 80, rows: 24, shell: null, workingDir: null });
          const paneId = genId();
          setTabs((cur) => cur.map((tt) => tt.id === t.id ? { ...tt, root: { id: paneId, type: 'leaf', terminalId }, activePaneId: paneId } : tt));
        })();
        return t; // temporary; will be replaced in async callback
      }
      // choose a new active leaf (prefer first leaf)
      const pickFirstLeaf = (n: PaneNode): string => {
        if (n.type === 'leaf') return n.id;
        return pickFirstLeaf(n.children[0]);
      };
      return { ...t, root: newRoot, activePaneId: pickFirstLeaf(newRoot) };
    }));
  };

  // Create initial tab on app start
  useEffect(() => {
    if (tabs.length === 0) {
      createTabWithTerminal();
    }
  }, []);

  return (
    <div className="terminal-container">
      <div className="terminal-header">
        <div className="terminal-tabs">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={`terminal-tab ${tab.active ? 'active' : ''}`}
              onClick={() => switchTab(tab.id)}
            >
              {tab.title}
              <span
                style={{ marginLeft: '8px', cursor: 'pointer' }}
                onClick={(e) => {
                  e.stopPropagation();
                  closeTab(tab.id);
                }}
              >
                ×
              </span>
            </button>
          ))}
          <button className="terminal-tab" onClick={createTabWithTerminal}>
            +
          </button>
        </div>
        
        <div className="terminal-controls">
          <button
            className="terminal-control-btn"
            onClick={() => setShowAIPanel(!showAIPanel)}
            title="Toggle AI Assistant"
          >
            🤖
          </button>
          <button
            className="terminal-control-btn"
            onClick={() => setShowWorkflows(true)}
            title={`Workflows (${settings?.keybindings?.open_workflows || 'Ctrl+Shift+W'})`}
          >
            ⚡
          </button>
          <button
            className="terminal-control-btn"
            onClick={() => setShowSettings(true)}
            title="Settings"
          >
            ⚙
          </button>
          <button
            className="terminal-control-btn"
            onClick={() => setShowAdvanced(true)}
            title="Advanced Features (Ctrl+Shift+A)"
          >
            🚀
          </button>
          {activeTab && (
            <>
              <button
                className="terminal-control-btn"
                onClick={() => splitActivePane('vertical')}
                title="Split vertically (side by side)"
              >
                ⫶
              </button>
              <button
                className="terminal-control-btn"
                onClick={() => splitActivePane('horizontal')}
                title="Split horizontally (stack)"
              >
                ⫻
              </button>
              <button
                className="terminal-control-btn"
                onClick={closeActivePane}
                title="Close active pane"
              >
                ✖
              </button>
            </>
          )}
        </div>
      </div>

      <div className="terminal-viewport">
        {tabs.map((tab) => (
          <div key={tab.id} className="tab-viewport" style={{ display: tab.active ? 'block' : 'none' }}>
            <PaneLayout
              node={tab.root}
              activePaneId={tab.activePaneId}
              onFocus={focusPane}
              visible={tab.active}
            />
          </div>
        ))}
      </div>

      {showAIPanel && (
        <AIPanel
          terminalId={activeTab ? (findNode(activeTab.root, activeTab.activePaneId) as any)?.terminalId ?? null : null}
          onClose={() => setShowAIPanel(false)}
        />
      )}

      {showWorkflows && (
        <WorkflowsPanel
          terminalId={activeTab ? (findNode(activeTab.root, activeTab.activePaneId) as any)?.terminalId ?? null : null}
          visible={showWorkflows}
          onClose={() => setShowWorkflows(false)}
        />
      )}

      {showSettings && (
        <SettingsPanel
          visible={showSettings}
          onClose={() => setShowSettings(false)}
          onSaved={(s) => { setSettings(s); applyAppearance(s); }}
        />
      )}

      {showAdvanced && (
        <AdvancedFeatures
          terminalId={activeTab ? (findNode(activeTab.root, activeTab.activePaneId) as any)?.terminalId ?? '' : ''}
          visible={showAdvanced}
          onClose={() => setShowAdvanced(false)}
        />
      )}
    </div>
  );
}

export default App;
