import React, { useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface AIPanelProps {
  terminalId: string | null;
  onClose: () => void;
}

const AIPanel: React.FC<AIPanelProps> = ({ terminalId, onClose }) => {
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [result, setResult] = useState<string>('');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const runAI = async (task: 'generate' | 'explain' | 'suggest') => {
    if (task === 'generate' && !input.trim()) return;
    setIsLoading(true);
    setResult('');
    try {
      let text: string = '';
      if (task === 'generate') {
        text = await invoke<string>('ai_generate_command', { terminalId, userInput: input });
      } else if (task === 'explain') {
        text = await invoke<string>('ai_explain_error', { terminalId, errorText: null });
      } else {
        if (!terminalId) return;
        text = await invoke<string>('ai_suggest_next', { terminalId });
      }
      setResult(text);
    } catch (e) {
      setResult(String(e));
    } finally {
      setIsLoading(false);
    }
  };

  const execute = async () => {
    if (!terminalId || !result.trim()) return;
    // Try to extract first code-like line; otherwise send as-is
    const firstLine = result.split('\n').find(l => l.trim() && !l.trim().startsWith('#')) || result;
    await invoke('write_to_terminal', { terminalId, data: firstLine.trim() + '\r' });
  };

  return (
    <div className="ai-panel">
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
        <h3 style={{ fontSize: '14px', fontWeight: 'bold' }}>AI Assistant</h3>
        <button className="terminal-control-btn" onClick={onClose} title="Close AI Panel">×</button>
      </div>

      <div style={{ display: 'flex', gap: 8, marginBottom: 8 }}>
        <button className="terminal-control-btn" onClick={() => runAI('generate')} title="Generate command from prompt" disabled={isLoading}>✨ Generate</button>
        <button className="terminal-control-btn" onClick={() => runAI('explain')} title="Explain last error/output" disabled={isLoading}>💡 Explain</button>
        <button className="terminal-control-btn" onClick={() => runAI('suggest')} title="Suggest next steps" disabled={isLoading || !terminalId}>🧭 Suggest</button>
        <button className="terminal-control-btn" onClick={execute} title="Run the result" disabled={!terminalId || !result.trim()}>▶ Run</button>
      </div>

      <textarea
        ref={textareaRef}
        className="ai-input"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder="Describe what you want to do..."
        rows={3}
        disabled={isLoading}
      />

      {isLoading && <div style={{ fontSize: 12, color: '#9aa5ce', marginTop: 8 }}>Processing...</div>}

      {result && (
        <pre style={{ marginTop: 8, background: '#0f1117', padding: 10, borderRadius: 6, whiteSpace: 'pre-wrap' }}>{result}</pre>
      )}
    </div>
  );
};

export default AIPanel;
