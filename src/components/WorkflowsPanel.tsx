import React, { useEffect, useMemo, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface WorkflowParam {
  name: string;
  description?: string;
  required: boolean;
  default?: string;
}

interface Workflow {
  id: string;
  name: string;
  description?: string;
  command: string;
  params: WorkflowParam[];
  tags: string[];
  created_at: number;
  updated_at: number;
}

interface Props {
  terminalId: string | null;
  visible: boolean;
  onClose: () => void;
}

const WorkflowsPanel: React.FC<Props> = ({ terminalId, visible, onClose }) => {
  const [list, setList] = useState<Workflow[]>([]);
  const [loading, setLoading] = useState(false);
  const [q, setQ] = useState('');
  const [selected, setSelected] = useState<Workflow | null>(null);
  const [values, setValues] = useState<Record<string, string>>({});
  const [showEditor, setShowEditor] = useState(false);
  const [editItem, setEditItem] = useState<Workflow | null>(null);

  const load = async () => {
    setLoading(true);
    try {
      const items = await invoke<Workflow[]>('list_workflows');
      setList(items);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { if (visible) load(); }, [visible]);

  const filtered = useMemo(() => {
    const s = q.toLowerCase();
    return list.filter(w => w.name.toLowerCase().includes(s) || w.tags.some(t => t.toLowerCase().includes(s)) || (w.description||'').toLowerCase().includes(s));
  }, [list, q]);

  const startRun = (wf: Workflow) => {
    setSelected(wf);
    const init: Record<string,string> = {};
    wf.params.forEach(p => { if (p.default) init[p.name] = p.default; });
    setValues(init);
  };

  const run = async () => {
    if (!terminalId || !selected) return;
    await invoke('run_workflow', { terminalId, workflowId: selected.id, values });
    setSelected(null);
    setValues({});
  };

  const del = async (wf: Workflow) => {
    await invoke('delete_workflow', { id: wf.id });
    load();
  };

  const save = async () => {
    if (!editItem) return;
    await invoke('save_workflow', { workflow: editItem });
    setShowEditor(false);
    setEditItem(null);
    load();
  };

  if (!visible) return null;

  return (
    <div className="workflows-overlay">
      <div className="workflows-modal">
        <div className="results-header">
          <div className="results-title">Workflows</div>
          <div className="results-actions">
            <input className="search-input" placeholder="Search workflows..." value={q} onChange={e => setQ(e.target.value)} />
            <button className="terminal-control-btn" onClick={() => { setShowEditor(true); setEditItem({ id: '', name: 'New Workflow', description: '', command: '', params: [], tags: [], created_at: 0, updated_at: 0 }); }}>＋ New</button>
            <button className="close-button" onClick={onClose}>×</button>
          </div>
        </div>
        <div className="results-content">
          <div className="results-list">
            {loading && <div className="history-loading">Loading...</div>}
            {!loading && filtered.map(w => (
              <div key={w.id} className="results-item" onClick={() => startRun(w)}>
                <div className="results-line-index">{w.tags.join(', ')}</div>
                <div className="results-line-text"><strong>{w.name}</strong> — {w.description}</div>
                <div style={{ marginTop: 4, fontFamily: 'Fira Code, monospace', color: '#9ca3af' }}>{w.command}</div>
                <div style={{ marginTop: 6, display: 'flex', gap: 8 }}>
                  <button className="terminal-control-btn" onClick={(e) => { e.stopPropagation(); setShowEditor(true); setEditItem(w); }}>Edit</button>
                  <button className="terminal-control-btn" onClick={(e) => { e.stopPropagation(); del(w); }}>Delete</button>
                  <button className="terminal-control-btn" onClick={(e) => { e.stopPropagation(); startRun(w); }}>Run</button>
                </div>
              </div>
            ))}
          </div>
          <div className="results-preview">
            <div style={{ padding: 12 }}>
              {selected ? (
                <div>
                  <div style={{ marginBottom: 8 }}><strong>{selected.name}</strong></div>
                  {selected.params.length > 0 ? (
                    <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                      {selected.params.map(p => (
                        <div key={p.name}>
                          <label style={{ display: 'block', fontSize: 12, color: '#9ca3af' }}>{p.name}{p.required ? ' *' : ''}</label>
                          <input className="search-input" value={values[p.name] || ''} onChange={e => setValues(v => ({ ...v, [p.name]: e.target.value }))} placeholder={p.description || ''} />
                        </div>
                      ))}
                      <div style={{ display: 'flex', gap: 8 }}>
                        <button className="terminal-control-btn" onClick={run} disabled={!terminalId}>Run in Terminal</button>
                        <button className="terminal-control-btn" onClick={() => { setSelected(null); setValues({}); }}>Cancel</button>
                      </div>
                    </div>
                  ) : (
                    <div>
                      <div style={{ marginBottom: 8, color: '#9ca3af' }}>No parameters needed.</div>
                      <button className="terminal-control-btn" onClick={run} disabled={!terminalId}>Run</button>
                    </div>
                  )}
                </div>
              ) : (
                <div style={{ color: '#9ca3af' }}>Select a workflow to run, or create a new one.</div>
              )}
            </div>
          </div>
        </div>
      
        {showEditor && editItem && (
          <div className="workflows-editor">
            <div className="results-header">
              <div className="results-title">Edit Workflow</div>
              <div className="results-actions">
                <button className="terminal-control-btn" onClick={() => { setShowEditor(false); setEditItem(null); }}>Close</button>
              </div>
            </div>
            <div className="editor-body">
              <input className="search-input" value={editItem.name} onChange={e => setEditItem({ ...editItem, name: e.target.value })} placeholder="Name" />
              <input className="search-input" value={editItem.description || ''} onChange={e => setEditItem({ ...editItem, description: e.target.value })} placeholder="Description" />
              <input className="search-input" value={(editItem.tags || []).join(', ')} onChange={e => setEditItem({ ...editItem, tags: e.target.value.split(',').map(s => s.trim()).filter(Boolean) })} placeholder="tags (comma separated)" />
              <textarea className="ai-input" rows={4} value={editItem.command} onChange={e => setEditItem({ ...editItem, command: e.target.value })} placeholder="Command (use {{param}} syntax)" />
              <div style={{ marginTop: 8 }}>
                <div style={{ fontSize: 12, marginBottom: 6, color: '#9aa3af' }}>Parameters</div>
                {(editItem.params || []).map((p, idx) => (
                  <div key={p.name+idx} style={{ display: 'flex', gap: 6, marginBottom: 6 }}>
                    <input className="search-input" style={{ flex: 1 }} value={p.name} onChange={e => {
                      const arr = [...editItem.params];
                      arr[idx] = { ...p, name: e.target.value };
                      setEditItem({ ...editItem, params: arr });
                    }} placeholder="name" />
                    <input className="search-input" style={{ flex: 2 }} value={p.description || ''} onChange={e => {
                      const arr = [...editItem.params];
                      arr[idx] = { ...p, description: e.target.value };
                      setEditItem({ ...editItem, params: arr });
                    }} placeholder="description" />
                    <input className="search-input" style={{ flex: 1 }} value={p.default || ''} onChange={e => {
                      const arr = [...editItem.params];
                      arr[idx] = { ...p, default: e.target.value };
                      setEditItem({ ...editItem, params: arr });
                    }} placeholder="default" />
                    <label className="search-toggle"><input type="checkbox" checked={p.required} onChange={e => {
                      const arr = [...editItem.params];
                      arr[idx] = { ...p, required: e.target.checked };
                      setEditItem({ ...editItem, params: arr });
                    }} /> req</label>
                    <button className="terminal-control-btn" onClick={() => {
                      const arr = [...editItem.params];
                      arr.splice(idx,1);
                      setEditItem({ ...editItem, params: arr });
                    }}>✖</button>
                  </div>
                ))}
                <button className="terminal-control-btn" onClick={() => setEditItem({ ...editItem, params: [...(editItem.params||[]), { name: 'param', description: '', required: false, default: '' }] })}>＋ Add param</button>
              </div>
              <div style={{ marginTop: 8 }}>
                <button className="terminal-control-btn" onClick={save}>Save</button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default WorkflowsPanel;
